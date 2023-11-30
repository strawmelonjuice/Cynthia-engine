use actix_files as fs;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use colored::Colorize;


fn logger(act: i8, msg: String) {
    /*
    
    Acts:
    0: Debug log, only act if logging is set to verbose
    1: General log item -- '[log]'
    2: Request that an Cynthia's part succeeded -- '[CYNGET/OK]'

    
     */
    let spaces: usize = 15;
    let tabs: String = format!("\t\t");
    if act == 1 {
        let name = "[Log]".blue();
        let spaceleft = if name.chars().count() < spaces {spaces - name.chars().count()} else { 0 };
        let title = format!("{}", name.bold().blue());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg);
    }
    if act == 2 {
        let name = "[CYNGET/OK] ✅";
        let spaceleft = if name.chars().count() < spaces {spaces - name.chars().count()} else { 0 };
        let title = format!("{}", name.bold().yellow());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg);
    }
}




#[derive(Deserialize, Debug, Serialize)]
struct CynthiaUrlDataF {
    fullurl: String,
}

#[get("/p/{id:.*}")]
async fn serves_p(id: web::Path<String>) -> HttpResponse {
    logger(2, format!("--> ID: '{}' was requested.", id));
    return HttpResponse::Ok().body(format!("User detail: {}", id));
}

async fn root() -> impl Responder {
    logger(2, format!("--> Home"));
    return HttpResponse::Ok().body(return_content_p("root".to_string(), "/".to_string()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let portnum: u16 = 3000;
    logger(1, format!("{}", format!("Starting server on {0}{1}", "http://localhost:".green(), portnum.to_string().bold().green()).italic()));
    HttpServer::new(|| {
        App::new()
        .service(fs::Files::new("/assets", "./assets").show_files_listing())
        .service(serves_p)
        .route("/", web::get().to(root))
    })
    .bind(("127.0.0.1", portnum))?
    .run()
    .await
}

fn return_content_p (id: String, probableurl: String) -> String {
    return "temporary".to_string();
}
