#[derive(clap::Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub short_session_time: usize,

    #[clap(long, env)]
    pub email_token_time: usize,

    #[clap(long, env)]
    pub long_session_time: usize,

    #[clap(long, env)]
    pub email_key: String,

    #[clap(long, env)]
    pub email_verification_template_key: String,

    #[clap(long, env)]
    pub email_reset_password_template_key: String,

    #[clap(long, env)]
    pub host: String,

    #[clap(long, env)]
    pub email_service_url: String,

    #[clap(long, env)]
    pub email_service_url_template: String,

    #[clap(long, env)]
    pub email_sender_address: String,

    #[clap(long, env)]
    pub company: String,
}
