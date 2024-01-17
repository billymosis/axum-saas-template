/// The configuration parameters for the application.
///
/// These can either be passed on the command line, or pulled from environment variables.
/// The latter is preferred as environment variables are one of the recommended ways to
/// get configuration from Kubernetes Secrets in deployment.
///
/// This is a pretty simple configuration struct as far as backend APIs go. You could imagine
/// a bunch of other parameters going here, like API keys for external services
/// or flags enabling or disabling certain features or test modes of the API.
///
/// For development convenience, these can also be read from a `.env` file in the working
/// directory where the application is started.
///
/// See `.env.sample` in the repository root for details.
#[derive(clap::Parser, Clone)]
pub struct Config {
    /// The connection URL for the Postgres database this application should use.
    #[clap(long, env)]
    pub database_url: String,

    /// The HMAC signing and verification key used for login tokens (JWTs).
    ///
    /// There is no required structure or format to this key as it's just fed into a hash function.
    /// In practice, it should be a long, random string that would be infeasible to brute-force.
    #[clap(long, env)]
    pub hmac_key: String,

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
