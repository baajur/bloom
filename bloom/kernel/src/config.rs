use std::{
    collections::{HashMap, HashSet},
    fmt,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::Error;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use stdx::{dotenv, encoding::base64};

const ENV_APP_ENV: &str = "APP_ENV";
const ENV_APP_BASE_URL: &str = "APP_BASE_URL";
const ENV_APP_MASTER_KEY: &str = "APP_MASTER_KEY";
const ENV_APP_OLD_MASTER_KEY: &str = "APP_OLD_MASTER_KEY";
const ENV_APP_SELF_HOSTED: &str = "APP_SELF_HOSTED";
const ENV_APP_DEBUG: &str = "APP_DEBUG";
const ENV_DATABASE_URL: &str = "DATABASE_URL";
const ENV_DATABASE_POOL_SIZE: &str = "DATABASE_POOL_SIZE";
const ENV_HTTP_PORT: &str = "PORT";
const ENV_HTTP_ACCESS_LOGS: &str = "HTTP_ACCESS_LOGS";
const ENV_HTTP_PUBLIC_DIRECTORY: &str = "HTTP_PUBLIC_DIRECTORY";
const ENV_MAIL_DRIVER: &str = "MAIL_DRIVER";
const ENV_MAIL_NOTIFY_ADDRESS: &str = "MAIL_NOTIFY_ADDRESS";
const ENV_MAIL_BLOCKLIST: &str = "MAIL_BLOCKLIST";
const ENV_STORAGE_DRIVER: &str = "STORAGE_DRIVER";
const ENV_STORAGE_BASE_DIRECTORY: &str = "STORAGE_BASE_DIRECTORY";
const ENV_STRIPE_SECRET_KEY: &str = "STRIPE_SECRET_KEY";
const ENV_STRIPE_PUBLIC_KEY: &str = "STRIPE_PUBLIC_KEY";
const ENV_STRIPE_WEBHOOK_SECRET: &str = "STRIPE_WEBHOOK_SECRET";
const ENV_STRIPE_DATA: &str = "STRIPE_DATA";
const ENV_AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
const ENV_AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
const ENV_AWS_DEFAULT_REGION: &str = "AWS_DEFAULT_REGION";
const ENV_SMTP_PORT: &str = "SMTP_PORT";
const ENV_SMTP_HOST: &str = "SMTP_HOST";
const ENV_SMTP_USERNAME: &str = "SMTP_USERNAME";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_S3_REGION: &str = "S3_REGION";
const ENV_S3_BUCKET: &str = "S3_BUCKET";
const ENV_SES_REGION: &str = "SES_REGION";
const ENV_WORKER_CONCURRENCY: &str = "WORKER_CONCURRENCY";
const ENV_SENTRY_SECURITY_REPORT_URI: &str = "SENTRY_SECURITY_REPORT_URI";
const ENV_SENTRY_INGEST_DOMAIN: &str = "SENTRY_INGEST_DOMAIN";
const ENV_SENTRY_DSN: &str = "SENTRY_DSN";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub env: String,
    pub base_url: String,
    pub master_key: Vec<u8>,
    pub old_master_key: Option<Vec<u8>>, // used for key rotation
    pub self_hosted: bool,
    pub debug: bool,
    // CountriesDataFile string `env:"APP_COUNTRIES_DATA" envDefault:"countries.json"`
    pub http: Http,
    pub database: Database,
    pub smtp: Smtp,
    pub mail: Mail,
    pub storage: Storage,
    pub stripe: Stripe,
    pub aws: Aws,
    pub ses: Ses,
    pub s3: S3,
    pub worker: Worker,
    pub sentry: Sentry,
}

const DEFAULT_APP_DEBUG: bool = false;
const DEFAULT_APP_SELF_HOSTED: bool = false;
const APP_ENV_DEV: &str = "dev";
const APP_ENV_STAGING: &str = "staging";
const APP_ENV_PRODUCTION: &str = "production";

/// Database contains the data necessary to connect to a database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool_size: u32,
}
const DEFAULT_DATABASE_POOL_SIZE: u32 = 100;

/// Http contains the data specific to the HTTP(s) server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http {
    pub port: u16,
    pub access_logs: bool,
    pub public_directory: String,
    // pub https_certs_directory: String,
    // pub https_certs_email: String,
    // pub https_domain: String,
    // pub https_port: u16,
}
const DEFAULT_HTTP_PORT: u16 = 8000;
const DEFAULT_ACCESS_LOGS: bool = false;
const DEFAULT_HTTP_PUBLIC_DIRECTORY: &str = "public";
// const ENV_HTTPS_CERTS_DIRECTORY: &str = "ENV_HTTPS_CERTS_DIRECTORY";
// const ENV_HTTPS_CERTS_EMAIL: &str = "HTTPS_CERTS_EMAIL";
// const ENV_HTTPS_DOMAIN: &str = "HTTPS_DOMAIN";
// const ENV_HTTPS_PORT: &str = "HTTPS_PORT";
// const DEFAULT_HTTPS_CERT_DIRECTORY: &str = "certs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mail {
    pub driver: MailDriver,
    pub notify_address: String,
    pub domains_blocklist_file: String,
    pub domains_blocklist: HashSet<String>,
    // 	OutboundAddress  mail.Address `env:"MAIL_OUTBOUND_ADDRESS"`
}
const DEFAULT_MAIL_BLOCKLIST_FILE: &str = "email_domains_blocklist.txt";
const DEFAULT_MAIL_DRIVER: MailDriver = MailDriver::Ses;
const MAIL_DRIVER_SES: &str = "ses";

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailDriver {
    Ses,
}

impl FromStr for MailDriver {
    type Err = Error;

    fn from_str(s: &str) -> Result<MailDriver, Error> {
        match s {
            MAIL_DRIVER_SES => Ok(MailDriver::Ses),
            _ => Err(Error::InvalidArgument(format!(
                "config: {} is not a valid mail driver. Valid values are [{}]",
                s,
                MailDriver::Ses,
            ))),
        }
    }
}

impl fmt::Display for MailDriver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MailDriver::Ses => write!(f, "{}", MAIL_DRIVER_SES),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub driver: StorageDriver,
    pub base_direcotry: String,
}
const DEFAULT_STORAGE_DRIVER: StorageDriver = StorageDriver::S3;
const DEFAULT_STORAGE_BASE_DIRECTORY: &str = "";
const STORAGE_DRIVER_S3: &str = "s3";

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageDriver {
    S3,
}

impl FromStr for StorageDriver {
    type Err = Error;

    fn from_str(s: &str) -> Result<StorageDriver, Error> {
        match s {
            STORAGE_DRIVER_S3 => Ok(StorageDriver::S3),
            _ => Err(Error::InvalidArgument(format!(
                "config: {} is not a valid storage driver. Valid values are [{}]",
                s,
                StorageDriver::S3,
            ))),
        }
    }
}

impl fmt::Display for StorageDriver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StorageDriver::S3 => write!(f, "{}", STORAGE_DRIVER_S3),
        }
    }
}

/// Stripe contains the data to connect to Stripe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stripe {
    pub secret_key: String,
    pub public_key: String,
    pub webhook_secret: String,
    pub data: StripeData,
    json_data: String,
    // StarterPlanID string `env:"STRIPE_STARTER_PLAN"`
    // ProPlanID     string `env:"STRIPE_PRO_PLAN"`
    // UltraPlanID   string `env:"STRIPE_ULTRA_PLAN"`
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeData {
    pub taxes: HashMap<String, String>,
    pub products: StripeProducts,
    pub prices: StripePrices,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeProducts {
    pub starter: String,
    pub pro: String,
    pub ultra: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripePrices {
    pub starter: String,
    pub pro: String,
    pub ultra: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aws {
    pub secret_access_key: Option<String>,
    pub access_key_id: Option<String>,
    pub default_region: String,
}
const DEFAULT_AWS_REGION: &str = "eu-central-1"; // Ireland

/// Smtp contains the data necessary to send emails using the SMTP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Smtp {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ses {
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3 {
    pub region: String,
    pub bucket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub concurrency: usize,
}
const DEFAULT_WORKER_CONCURRENCY: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentry {
    pub security_report_uri: Option<String>,
    pub ingest_domain: Option<String>,
    pub dsn: Option<String>,
}

impl Config {
    /// Load and validate the configuration from the environment.
    /// If an error is found while parsing the values, or validating the data, an error is returned.
    pub fn load() -> Result<Config, Error> {
        dotenv::dotenv().ok();

        // app
        let env = std::env::var(ENV_APP_ENV).map_err(|_| env_not_found(ENV_APP_ENV))?;
        let base_url = std::env::var(ENV_APP_BASE_URL).map_err(|_| env_not_found(ENV_APP_BASE_URL))?;
        let master_key = std::env::var(ENV_APP_MASTER_KEY)
            .map_err(|_| env_not_found(ENV_APP_MASTER_KEY))
            .map(base64::decode)??;
        let old_master_key = std::env::var(ENV_APP_OLD_MASTER_KEY)
            .ok()
            .map_or(Ok(None), |env_val| base64::decode(env_val).map(Some))?;
        let self_hosted = std::env::var(ENV_APP_SELF_HOSTED)
            .ok()
            .map_or(Ok(DEFAULT_APP_SELF_HOSTED), |env_val| env_val.parse::<bool>())?;
        let debug = std::env::var(ENV_APP_DEBUG)
            .ok()
            .map_or(Ok(DEFAULT_APP_DEBUG), |env_val| env_val.parse::<bool>())?;

        // http
        let http_port = std::env::var(ENV_HTTP_PORT)
            .ok()
            .map_or(Ok(DEFAULT_HTTP_PORT), |env_val| env_val.parse::<u16>())?;
        let http_access_logs = std::env::var(ENV_HTTP_ACCESS_LOGS)
            .ok()
            .map_or(Ok(DEFAULT_ACCESS_LOGS), |env_val| env_val.parse::<bool>())?;
        let http_public_directory =
            std::env::var(ENV_HTTP_PUBLIC_DIRECTORY).unwrap_or(String::from(DEFAULT_HTTP_PUBLIC_DIRECTORY));
        // let https_certs_directory =
        //     std::env::var(ENV_HTTPS_CERTS_DIRECTORY).unwrap_or(String::from(DEFAULT_HTTPS_CERT_DIRECTORY));
        // let https_certs_email = std::env::var(ENV_HTTPS_CERTS_EMAIL).unwrap_or(String::new());
        // let https_domain = std::env::var(ENV_HTTPS_DOMAIN).unwrap_or(String::new());
        // let https_port = std::env::var(ENV_HTTPS_PORT)
        //     .ok()
        //     .map_or(Ok(0), |env_val| env_val.parse::<u16>())?;

        let http = Http {
            port: http_port,
            access_logs: http_access_logs,
            public_directory: http_public_directory,
        };

        // database
        let database_url = std::env::var(ENV_DATABASE_URL).map_err(|_| env_not_found(ENV_DATABASE_URL))?;
        let database_pool_size = std::env::var(ENV_DATABASE_POOL_SIZE)
            .ok()
            .map_or(Ok(DEFAULT_DATABASE_POOL_SIZE), |pool_size_str| {
                pool_size_str.parse::<u32>()
            })?;

        let database = Database {
            url: database_url,
            pool_size: database_pool_size,
        };

        // smtp
        let smtp_port = std::env::var(ENV_SMTP_PORT)
            .ok()
            .map_or(Ok(None), |smtp_port_str| smtp_port_str.parse::<u16>().map(Some))?;
        let smtp_host = std::env::var(ENV_SMTP_HOST).ok();
        let smtp_username = std::env::var(ENV_SMTP_USERNAME).ok();
        let smtp_password = std::env::var(ENV_SMTP_PASSWORD).ok();

        let smtp = Smtp {
            port: smtp_port,
            host: smtp_host,
            username: smtp_username,
            password: smtp_password,
        };

        // mail
        let mail_driver = std::env::var(ENV_MAIL_DRIVER)
            .ok()
            .map_or(Ok(DEFAULT_MAIL_DRIVER), |env_val| env_val.parse::<MailDriver>())?;
        let mail_notify_address =
            std::env::var(ENV_MAIL_NOTIFY_ADDRESS).map_err(|_| env_not_found(ENV_MAIL_NOTIFY_ADDRESS))?;
        let mail_domains_blocklist_file =
            std::env::var(ENV_MAIL_BLOCKLIST).unwrap_or(String::from(DEFAULT_MAIL_BLOCKLIST_FILE));

        let mail_domains_blocklist: HashSet<String> = {
            let blocklist_file = File::open(&mail_domains_blocklist_file)?;
            let reader = BufReader::new(&blocklist_file);
            let mut blocklist = HashSet::new();
            for line in reader.lines() {
                let line = line?.trim().to_string();
                blocklist.insert(line);
            }
            blocklist
        };

        let mail = Mail {
            driver: mail_driver,
            notify_address: mail_notify_address,
            domains_blocklist_file: mail_domains_blocklist_file,
            domains_blocklist: mail_domains_blocklist,
        };

        // storage
        let storage_driver = std::env::var(ENV_STORAGE_DRIVER)
            .ok()
            .map_or(Ok(DEFAULT_STORAGE_DRIVER), |env_val| env_val.parse::<StorageDriver>())?;
        let storage_base_direcotry =
            std::env::var(ENV_STORAGE_BASE_DIRECTORY).unwrap_or(String::from(DEFAULT_STORAGE_BASE_DIRECTORY));

        let storage = Storage {
            driver: storage_driver,
            base_direcotry: storage_base_direcotry,
        };

        let stripe_private_key =
            std::env::var(ENV_STRIPE_SECRET_KEY).map_err(|_| env_not_found(ENV_STRIPE_SECRET_KEY))?;
        let stripe_public_key =
            std::env::var(ENV_STRIPE_PUBLIC_KEY).map_err(|_| env_not_found(ENV_STRIPE_SECRET_KEY))?;
        let stripe_webhook_secret =
            std::env::var(ENV_STRIPE_WEBHOOK_SECRET).map_err(|_| env_not_found(ENV_STRIPE_SECRET_KEY))?;
        let stripe_data_json = std::env::var(ENV_STRIPE_DATA).map_err(|_| env_not_found(ENV_STRIPE_SECRET_KEY))?;
        let stripe_data: StripeData = serde_json::from_str(&stripe_data_json)?;

        let stripe = Stripe {
            secret_key: stripe_private_key,
            public_key: stripe_public_key,
            webhook_secret: stripe_webhook_secret,
            data: stripe_data,
            json_data: stripe_data_json,
        };

        // aws
        let aws_secret_access_key = std::env::var(ENV_AWS_SECRET_ACCESS_KEY).ok();
        let aws_access_key_id = std::env::var(ENV_AWS_ACCESS_KEY_ID).ok();
        let aws_default_region = std::env::var(ENV_AWS_DEFAULT_REGION).unwrap_or(String::from(DEFAULT_AWS_REGION));

        let aws = Aws {
            secret_access_key: aws_secret_access_key,
            access_key_id: aws_access_key_id,
            default_region: aws_default_region,
        };

        // ses
        let ses_region = std::env::var(ENV_SES_REGION).unwrap_or(aws.default_region.clone());

        let ses = Ses { region: ses_region };

        // s3
        let s3_region = std::env::var(ENV_S3_REGION).unwrap_or(aws.default_region.clone());
        let s3_bucket = std::env::var(ENV_S3_BUCKET).ok();

        let s3 = S3 {
            region: s3_region,
            bucket: s3_bucket,
        };

        // worker
        let worker_concurrency = std::env::var(ENV_WORKER_CONCURRENCY)
            .ok()
            .map_or(Ok(DEFAULT_WORKER_CONCURRENCY), |env_val| env_val.parse::<usize>())?;

        let worker = Worker {
            concurrency: worker_concurrency,
        };

        // sentry
        let sentry_security_report_uri = std::env::var(ENV_SENTRY_SECURITY_REPORT_URI).ok();
        let sentry_ingest_domain = std::env::var(ENV_SENTRY_INGEST_DOMAIN).ok();
        let sentry_dsn = std::env::var(ENV_SENTRY_DSN).ok();

        let sentry = Sentry {
            security_report_uri: sentry_security_report_uri,
            ingest_domain: sentry_ingest_domain,
            dsn: sentry_dsn,
        };

        let mut config = Config {
            env,
            base_url,
            master_key,
            old_master_key,
            self_hosted,
            debug,
            http,
            database,
            smtp,
            mail,
            storage,
            stripe,
            aws,
            ses,
            s3,
            worker,
            sentry,
        };

        config.clean_and_validate()?;

        Ok(config)
    }

    fn clean_and_validate(&mut self) -> Result<(), Error> {
        // app
        match self.env.as_str() {
            APP_ENV_DEV | APP_ENV_STAGING | APP_ENV_PRODUCTION => Ok(()),
            env => Err(Error::InvalidArgument(format!("config: Invalid env: {}", env))),
        }?;
        Ok(())
    }
}

fn env_not_found(var: &str) -> Error {
    Error::NotFound(format!("config: {} env var not found", var))
}