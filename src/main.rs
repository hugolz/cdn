// #[macro_use]
// extern crate rocket;
#[allow(unused_imports)]
#[macro_use(trace, debug, info, warn, error)]
extern crate log;

mod basic;
mod json;

#[rocket::get("/<id>")]
async fn download(id: &str) -> String {
    format!("Could not fetch {id}, this part is not done yet :)")
}

#[rocket::get("/")]
fn root() -> &'static str {
    "
        Hi, please take a look at the /examples directory to understand how to use this api
    "
}

#[rocket::catch(403)]
pub async fn root_403() -> String {
    "403".to_string()
}

#[rocket::main]
async fn main() {
    let logcfg = logger::LoggerConfig::new()
        .set_level(log::LevelFilter::Trace)
        .add_filter("rocket", log::LevelFilter::Warn);
    logger::init(logcfg, Some("log/server.log"));

    let width = 30;
    let message = "Program start";
    let chr = "─";

    // Small print to show the start of the program log in the file
    trace!(
        "\n╭{line}╮\n│{left_spaces}{text}{right_spaces}{conditional_space}│\n╰{line}╯",
        line = chr.repeat(width),
        left_spaces = " ".repeat((width - message.len()) / 2),
        text = message,
        conditional_space = " ".repeat(if width % 2 == 0 { 1 } else { 0 }),
        right_spaces = " ".repeat((width - message.len()) / 2),
    );

    // let rocket = rocket::build();

    let rocket = rocket::build()
        .attach(rocket::fairing::AdHoc::on_liftoff(
            "log config",
            |_rocket_orbit| {
                std::boxed::Box::pin(async move {
                    debug!("Hi"); // Was used to do some tests      
                })
            },
        ))
        .register("/", rocket::catchers![root_403])
        .register("/json", rocket::catchers![json::upload_json_400])
        .mount(
            "/",
            rocket::routes![root, json::upload_json, basic::upload, download],
        )
        .ignite()
        .await
        .unwrap();

    /*----------------------------

        Display the startup data

    ----------------------------*/
    display_config(rocket.config(), rocket.routes(), rocket.catchers());

    rocket.launch().await.unwrap();
}

fn display_config<'a>(
    rocket_cfg: &rocket::Config,
    rocket_routes: impl Iterator<Item = &'a rocket::Route>,
    rocket_catchers: impl Iterator<Item = &'a rocket::Catcher>,
) {
    let profile = rocket_cfg.profile.as_str().as_str();
    let address = rocket_cfg.address;
    let port = rocket_cfg.port;
    let workers = rocket_cfg.workers;
    // let max_blocking = cfg.max_blocking;
    let indent = rocket_cfg.ident.as_str().unwrap_or("[ERROR] Undefined");
    let ip_headers = rocket_cfg
        .ip_header
        .as_ref().map(|header| header.as_str())
        .unwrap_or("[ERROR] Undefined");
    let limits = ["bytes", "data-form", "file", "json", "msgpack", "string"]
        .iter()
        .map(|limit_name| {
            format!(
                "{limit_name}: {}",
                rocket_cfg
                    .limits
                    .get(limit_name)
                    .unwrap_or(rocket::data::ByteUnit::from(0))
            )
        })
        .collect::<Vec<String>>();
    let keep_alive_s = rocket_cfg.keep_alive;
    let shutdown_mode = &rocket_cfg.shutdown;

    let routes = rocket_routes
        .map(|route| {
            let uri = route.uri.origin.to_string();
            let name = route
                .name
                .as_ref()
                .map(|name| name.as_ref())
                .unwrap_or("[ERROR] Undefined");
            let method = route.method.as_str();
            format!("({name}) {uri} {method}")
        })
        .collect::<Vec<String>>();

    let catchers = rocket_catchers
        .map(|catcher| {
            let base = catcher.base.to_string();
            let name = catcher
                .name
                .as_ref()
                .map(|name| name.as_ref())
                .unwrap_or("[ERROR] Undefined");
            let code = catcher
                .code
                .map(|code| code.to_string())
                .unwrap_or("[ERROR] Undefined".to_string());

            format!("({name}) {base} {code}")
        })
        .collect::<Vec<String>>();

    let display_vec = |data: Vec<String>| -> String {
        let mut out = String::new();
        out.push_str("[\n");
        for d in data {
            out.push_str(&format!(" {d}\n"))
        }
        out.push(']');
        out
    };
    debug!("Config:\nUsing profile: {profile}\nAddress: {address}:{port}\nWorkers: {workers}\nIndent: {indent}\nHeaders: {ip_headers}\nLimits: {formatted_limits}\nConnection lifetime: {keep_alive_s}s\nShutdown mode: {shutdown_mode}\nRoutes: {formatted_routes}\nCatchers: {formatted_catchers}",
        formatted_limits = display_vec(limits),
        formatted_routes = display_vec(routes),
        formatted_catchers = display_vec(catchers)
    );
}
