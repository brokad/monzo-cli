#![feature(async_closure)]

#[macro_use]
extern crate anyhow;

use std::pin::Pin;

use anyhow::Result;

use structopt::StructOpt;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use std::future::Future;

use console::{pad_str, strip_ansi_codes, style};

pub mod config;
pub use crate::config::Config;

#[derive(StructOpt)]
#[structopt(name = "monzo-cli")]
#[structopt(about = "Monzo on the command-line")]
pub struct Args {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(about = "List all accounts and their balance")]
    Accounts,
    #[structopt(name = "ls")]
    #[structopt(about = "List transactions")]
    List,
}

impl Args {
    pub fn from_args() -> Self {
        <Self as StructOpt>::from_args()
    }
}

pub struct Cli<W = tokio::io::Stdout> {
    args: Args,
    config: Config,
    writer: W,
}

impl Cli {
    pub fn new_stdout(args: Args, config: Config) -> Cli {
        Cli::new(args, config, tokio::io::stdout())
    }
}

impl<W> Cli<W> {
    pub fn new(args: Args, config: Config, writer: W) -> Self {
        Self {
            args,
            config,
            writer,
        }
    }

    fn format_amount(amount: i64) -> String {
        let major = amount / 100;
        let minor = amount.abs() % 100;
        format!("{}.{}", major, minor)
    }

    fn align(content: &str, len: usize) -> std::borrow::Cow<'_, str> {
        pad_str(content, len, console::Alignment::Left, None)
    }
}

trait AsyncWriteFmtExt
where
    Self: AsyncWrite,
{
    fn write_fmt<'a>(
        &'a mut self,
        args: std::fmt::Arguments<'a>,
    ) -> Pin<Box<dyn Future<Output = tokio::io::Result<()>> + 'a>>
    where
        Self: Unpin,
    {
        Box::pin(async move { self.write_all(args.to_string().as_bytes()).await })
    }
}

impl<T> AsyncWriteFmtExt for T where T: AsyncWrite {}

impl<W> Cli<W>
where
    W: AsyncWrite + Unpin,
{
    pub async fn run(mut self) -> Result<()> {
        let mut writer = Pin::new(&mut self.writer);

        let access_token = self.config.get_token()?;
        let client = monzo::Client::new(access_token);

        match self.args.command {
            Command::Accounts => {
                let accounts = client.accounts().await?;

                let balances_fut = accounts.iter().map(|account| client.balance(&account.id));
                let mut balances = futures::future::try_join_all(balances_fut)
                    .await?
                    .into_iter();

                let pots_fut = accounts.iter().map(|account| client.pots(&account.id));
                let mut all_pots = futures::future::try_join_all(pots_fut).await?.into_iter();

                for account in accounts {
                    let balance = balances.next().unwrap();
                    let pots = all_pots.next().unwrap();

                    // TODO: make `Type` public
                    let account_type = match format!("{:?}", account.account_type).as_str() {
                        "UkRetail" => "personal",
                        "UkRetailJoint" => "joint",
                        "UkBusiness" => "business",
                        _ => todo!("this type should be public upstream"),
                    };

                    let mut account_number = account.account_number;
                    let truncated_len = account_number.len() - 4;
                    let obfuscated = std::iter::repeat("*")
                        .take(truncated_len)
                        .collect::<String>();
                    account_number.replace_range(0..truncated_len, &obfuscated);

                    let account_line = format!(
                        "{} {} [{}]",
                        style("Account").bold().cyan(),
                        account_number,
                        account_type
                    );

                    let len = strip_ansi_codes(&account_line).len() + 1;

                    writeln!(writer, "{}", account_line).await?;

                    writeln!(
                        writer,
                        "{} {} {}",
                        Self::align("  Balance", len),
                        Self::format_amount(balance.balance),
                        balance.currency
                    )
                    .await?;

                    writeln!(
                        writer,
                        "{} {} {}",
                        Self::align("  Spent today", len),
                        Self::format_amount(-balance.spend_today),
                        balance.currency
                    )
                        .await?;

                    writeln!(writer, "").await?;

                    for pot in pots {
                        if pot.balance == 0 {
                            continue;
                        }

                        let pot_line = format!("  {} {}", style("Pot").bold().magenta(), pot.name);

                        writeln!(writer, "{}", Self::align(&pot_line, len)).await?;

                        writeln!(
                            writer,
                            "{} {} {}",
                            Self::align("    Balance", len),
                            Self::format_amount(pot.balance),
                            pot.currency
                        ).await?;

                        writeln!(writer, "").await?;
                    }
                }
            },
            Command::List => {
                todo!()
            },
        }

        Ok(())
    }
}
