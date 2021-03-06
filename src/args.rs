use app::{App, Args, Opt};
use toml;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::collections::HashMap as Map;
use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::env;
use std;

use super::consts::*; // 名字,版本,作者，简介，地址

/// Get `Config` by `parse` `args`
pub fn parse() -> Config {
    let mut config = Config::default();
    let mut server = Server::default();
    let mut routes: Vec<String> = vec!["./".to_owned()];
    let mut cp = false;
    let mut c_path: Option<String> = None;
    let mut redirect_html = false;

    let helper = {
        App::new(NAME)
            .version(VERSION)
            .author(AUTHOR, EMAIL)
            .addr(URL_NAME, URL)
            .desc(DESC)
            .opt(
                Opt::new("cp", &mut cp)
                    .short('C')
                    .long("config-print")
                    .help("Print the default config file"),
            )
            .opt(
                Opt::new("config", &mut c_path)
                    .optional()
                    .short('c')
                    .long("config")
                    .help("Sets a custom config file"),
            )
            .opt(
                Opt::new("bbredircet-html", &mut redirect_html)
                    .short('r')
                    .long("redirect-html")
                    .help("Redirect dir to `index.html/htm`, if it exists"),
            )
            .opt(
                Opt::new("keepalive", &mut config.keep_alive)
                    .sort_key("cka")
                    .short('k')
                    .long("keep-alive")
                    .help("Close HTTP keep alive"),
            )
            .opt(
                Opt::new("follow-links", &mut config.follow_links)
                    .short('f')
                    .long("follow-links")
                    .help("Whether follow links(default follow)"),
            )
            .opt(
                Opt::new("byte", &mut config.magic_limit)
                    .short('m')
                    .long("magic-limit")
                    .help("The limit for detect file ContenType(use 0 to close)"),
            )
            .opt(
                Opt::new("secs", &mut config.cache_secs)
                    .sort_key("csecs")
                    .short('s')
                    .long("cache-secs")
                    .help("Sets cache secs(use 0 to close)"),
            )
            .opt(
                Opt::new("ip", &mut server.ip)
                    .short('i')
                    .long("ip")
                    .help("Sets listenning ip"),
            )
            .opt(
                Opt::new("port", &mut server.port)
                    .short('p')
                    .long("port")
                    .help("Sets listenning port"),
            )
            .args(Args::new("PATH", &mut routes).help(r#"Sets the paths to share"#))
            .parse_args()
    };
    // -cp/--cp
    if cp {
        config_print();
    }
    //-c/--config选项，如果有就载入该文件。
    if let Some(s) = c_path {
        return Config::load_from_file(&s)
            .map_err(|e| helper.help_err_exit(e, 1))
            .unwrap();
    }
    // 命令行有没有参数？有就解析参数，没有就寻找配置文件，再没有就使用默认配置。
    if env::args().skip(1).len() == 0 {
        match get_config_path() {
            Some(s) => Config::load_from_file(&s)
                .map_err(|e| helper.help_err_exit(e, 1))
                .unwrap(),
            None => Config::load_from_STR(),
        }
    } else {
        config.addrs.clear();
        config.addrs.push(SocketAddr::new(server.ip, server.port));
        config.routes = args_paths_to_route(&routes[..], redirect_html)
            .map_err(|e| helper.help_err_exit(e, 1))
            .unwrap();
        config
    }
}

#[derive(Debug, Clone)]
struct Server {
    pub ip: IpAddr,
    pub port: u16,
}
impl Default for Server {
    fn default() -> Server {
        Self::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080)
    }
}
impl Server {
    fn new(ip: IpAddr, port: u16) -> Self {
        Server { ip: ip, port: port }
    }
}
// 关键是结构体的字段名，和toml的[name]对应
#[derive(Debug, Deserialize)]
struct Fht2p {
    setting: Setting,
    routes: Vec<Route>,
}

#[derive(Debug, Deserialize)]
struct Setting {
    #[serde(rename = "keep-alive")]
    keep_alive: bool,
    #[serde(rename = "magic-limit")]
    magic_limit: u64,
    #[serde(rename = "follow-links")]
    follow_links: bool,
    #[serde(rename = "cache-secs")]
    cache_secs: u32,
    addrs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Route {
    #[serde(default)]
    pub url_components: Vec<String>,
    pub url: String,
    pub path: String,
    #[serde(rename = "redirect-html")]
    pub redirect_html: bool,
}

impl Route {
    fn new<S: Into<String>>(url: S, path: S, redirect_html: bool) -> Self {
        Self {
            url_components: Vec::new(),
            url: url.into(),
            path: path.into(),
            redirect_html: redirect_html,
        }
    }
}

/// `Config` for `main`
#[derive(Debug)]
pub struct Config {
    pub keep_alive: bool,
    pub follow_links: bool,
    pub cache_secs: u32,
    pub magic_limit: u64,
    pub addrs: Vec<SocketAddr>,
    pub routes: Map<String, Route>,
}
impl Default for Config {
    fn default() -> Self {
        let mut map = Map::new();
        map.insert("/".to_owned(), Route::new("/", ".", false));
        Config {
            keep_alive: true,
            magic_limit: *MAGIC_LIMIT.get(),
            follow_links: true,
            cache_secs: 60,
            addrs: vec![
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            ],
            routes: map,
        }
    }
}

impl Config {
    fn load_from_file(path: &str) -> Result<Self, String> {
        let mut str = String::new();
        let mut file = File::open(path).map_err(|e| format!("config file('{}') open fails: {}", path, e.description()))?;
        file.read_to_string(&mut str)
            .map_err(|e| format!("config file('{}') read fails: {}", path, e.description()))?;
        Self::load_from_str(path, &str)
    }
    fn load_from_str(file_name: &str, toml: &str) -> Result<Config, String> {
        let mut config = Self::default();
        config.routes.clear();
        config.addrs.clear();

        let toml: Fht2p = toml::from_str(toml).map_err(|e| format!("config file('{}') parse fails: {}", file_name, e))?;
        config.keep_alive = toml.setting.keep_alive;
        config.magic_limit = toml.setting.magic_limit;
        config.follow_links = toml.setting.follow_links;
        config.cache_secs = toml.setting.cache_secs;
        for server in toml.setting.addrs {
            let addr = server.parse::<SocketAddr>().map_err(|e| {
                format!(
                    "config file('{}')'s {} parse::<SocketAddr> fails: {}",
                    file_name,
                    server,
                    e.description()
                )
            })?;
            config.addrs.push(addr);
        }

        for route in &toml.routes {
            if !Path::new(&route.path).exists() {
                warn!(
                    "'{}''s routes({:?}: {:?}) is not exists",
                    file_name, route.url, route.path
                );
            }
            if config
                .routes
                .insert(
                    route.url.clone(),
                    Route::new(route.url.as_str(), route.path.as_str(), route.redirect_html),
                )
                .is_some()
            {
                return Err(format!(
                    "'{}''s routes's {:?} already defined",
                    file_name, route.url
                ));
            }
        }
        if config.addrs.is_empty() {
            return Err(format!("'{}''s addrs is empty", file_name));
        }
        if config.routes.is_empty() {
            return Err(format!("'{}''s routes is empty", file_name));
        }
        Ok(config)
    }
    #[allow(non_snake_case)]
    fn load_from_STR() -> Self {
        Config::load_from_str("CONFIG-STR", CONFIG_STR).unwrap()
    }
}

// 打印默认配置文件。
fn config_print() {
    println!("{}", CONFIG_STR);
    std::process::exit(0);
}

fn get_config_path() -> Option<String> {
    match std::env::home_dir() {
        // 家目录 ～/.config/fht2p/fht2p.toml
        Some(ref home)
            if home.as_path()
                .join(".config/fht2p")
                .join(CONFIG_STR_PATH)
                .exists() =>
        {
            Some(
                home.as_path()
                    .join(".config/fht2p")
                    .join(CONFIG_STR_PATH)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
        // 可执行文件所在目录 path/fht2p.toml
        _ if std::env::current_exe().is_ok()
            && std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join(CONFIG_STR_PATH)
                .exists() =>
        {
            Some(
                std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join(CONFIG_STR_PATH)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
        // 当前目录 dir/fht2p.toml
        _ if std::env::current_dir().is_ok()
            && std::env::current_dir()
                .unwrap()
                .join(CONFIG_STR_PATH)
                .exists() =>
        {
            Some(
                std::env::current_dir()
                    .unwrap()
                    .join(CONFIG_STR_PATH)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
        _ => None,
    }
}

// 参数转换为Route url, path
fn args_paths_to_route(map: &[String], redirect_html: bool) -> Result<Map<String, Route>, String> {
    let mut routes = Map::new();
    for (idx, path) in map.iter().enumerate() {
        if !Path::new(&path).exists() {
            warn!("{:?} is not exists", &path);
        }
        if idx == 0 {
            let route = Route::new("/".to_owned(), path.to_string(), redirect_html);
            routes.insert("/".to_owned(), route);
        } else {
            let route_url = route_name(path)?;
            let route = Route::new(route_url.clone(), path.to_string(), redirect_html);
            if routes.insert(route_url, route).is_some() {
                return Err(format!("{} already defined", route_name(path).unwrap()));
            }
        }
    }
    fn route_name(msg: &str) -> Result<String, String> {
        let path = Path::new(msg);
        path.file_name()
            .map(|s| "/".to_owned() + s.to_str().unwrap())
            .map(|mut s| {
                if Path::new(msg).is_dir() {
                    s.push('/');
                }
                s
            })
            .ok_or_else(|| format!("Path '{}' dost not have name", msg))
    }
    Ok(routes)
}
