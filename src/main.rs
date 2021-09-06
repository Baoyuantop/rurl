use clap::{AppSettings, Clap};
use anyhow::{anyhow, Result};
use reqwest::{header, Client, Response, Url};
use std::{collections::HashMap, str::FromStr};
use colored::*;
use mime::Mime;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Baoyuan <baoyuan.top@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
  Get(Get),
  Post(Post),
}

/// send a get request
#[derive(Clap, Debug)]
struct Get {
  #[clap(parse(try_from_str = parse_url))]
  url: String,
}

/// send a post request
#[derive(Clap, Debug)]
struct Post {
  #[clap(parse(try_from_str = parse_url))]
  url: String,
  #[clap(parse(try_from_str=parse_kv_pair))]
  body: Vec<KvPair>,
}

#[derive(Debug)]
struct KvPair {
  k: String,
  v: String,
}

fn parse_url(s: &str) -> Result<String> {
  let _url: Url = s.parse()?;
  Ok(s.into())
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Fail to parse {}", s));
        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let res = client.get(&args.url).send().await?;
    Ok(print_res(res).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let res = client.post(&args.url).json(&body).send().await?;
    Ok(print_res(res).await?)
}

fn print_status(res: &Response) {
    let status = format!("{:?} {}", res.version(), res.status()).blue();
    println!("{}\n", status);
}

fn print_header(res: &Response) {
    for (name, value) in res.headers() {
        println!("{} {:?}", name.to_string().yellow(), value)
    }
    println!("\n");
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan());
        },
        _ => println!("{}", body),
    }
}

fn get_content_type(res: &Response) -> Option<Mime> {
    res.headers().get(header::CONTENT_TYPE).map(|v| v.to_str().unwrap().parse().unwrap())
}

async fn print_res(res: Response) -> Result<()> {
    print_status(&res);
    print_header(&res);
    let mime = get_content_type(&res);
    let body = res.text().await?;
    print_body(mime, &body);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    
    let client = Client::new();
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };
    Ok(result)
}
