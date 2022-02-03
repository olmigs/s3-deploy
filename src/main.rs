#![allow(dead_code)]
use aws_sdk_s3::{ByteStream, Client, Error};
use clap::{Parser, Subcommand};
use mime::Mime;
use std::{collections::HashMap, fs, path::{Path, PathBuf}};

fn get_public_files(path: &str) -> Vec<String> {
    // get pathbuf
    let mut public_path = PathBuf::from(path);
    public_path.push("out");
    public_path.push("public.json");
    let contents = fs::read_to_string(public_path).expect("Could not find public.json");
    let json: Vec<String> = serde_json::from_str(&contents).expect("It's not an array somehow");
    json
}

fn get_modified_files(path: &String) -> HashMap<String, String> {
    let in_filenames = get_public_files(path);
    let mut out_filenames = HashMap::new();
    for name in in_filenames {
        let mut file_path = PathBuf::from(path);
        file_path.push("public");
        file_path.push(&name);
        let metadata = fs::metadata(file_path).expect("Error getting metadata");
        let last_modified = metadata
            .modified()
            .expect("Error getting system time")
            .elapsed()
            .expect("Error getting elapsed")
            .as_secs();
        let file_string = path.clone() + "/public/" + &name; // migsnote: use file_path.to_str?
        if last_modified < 24 * 3600 && metadata.is_file() {
            out_filenames.insert(name, file_string);
        }
    }
    out_filenames
}

fn get_mime_type(filename: &str) -> Result<Mime, &'static str> {
    let parts: Vec<&str> = filename.split('.').collect();
    let res = match parts.last() {
        Some(v) => match *v {
            "css" => mime::TEXT_CSS,
            "html" => mime::TEXT_HTML,
            "png" => mime::IMAGE_PNG,
            "js" => mime::APPLICATION_JAVASCRIPT,
            "jpg" => mime::IMAGE_JPEG,
            "json" => mime::APPLICATION_JSON,
            "map" => mime::APPLICATION_OCTET_STREAM,
            "svg" => mime::IMAGE_SVG,
            &_ => mime::TEXT_PLAIN,
        },
        None => mime::TEXT_PLAIN,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_out_files_len() {
        let files = get_public_files("/Static/site/assets");
        assert_eq!(files.len(), 7);
    }

    #[test]
    fn test_mime_types() {
        let files = get_public_files("/Static/site/assets");
        for file in files {
            let res = get_mime_type(&file);
            assert_eq!(res.is_ok(), true);
        }
    }
}

async fn show_objects(client: &Client, bucket: &str) -> Result<(), Error> {
    let resp = client.list_objects_v2().bucket(bucket).send().await?;
    println!("{}", bucket);
    for object in resp.contents().unwrap_or_default() {
        println!("   {}", object.key().unwrap_or_default());
    }
    Ok(())
}

async fn upload_object(
    client: &Client,
    bucket: &str,
    mime: mime::Mime,
    filename: &str,
    key: &str,
) -> Result<(), Error> {
    let body = ByteStream::from_path(Path::new(filename)).await;
    match body {
        Ok(b) => {
            let resp = client
                .put_object()
                .content_type(mime.to_string())
                .bucket(bucket)
                .key(key)
                .body(b)
                .send()
                .await?;
            match resp.e_tag {
                Some(tag) => println!("Upload success for {} \n   Entity tag {}", key, tag),
                None => println!("Upload success for {}", key),
            }
        }
        Err(e) => {
            println!("Got an error uploading {} : {}", key, e);
        }
    }
    Ok(())
}

async fn print(bucket: &String, client: &Client) -> () {
    match show_objects(client, bucket).await {
        Err(e) => println!("{}", e),
        _ => {}
    }
}

async fn deploy(
    bucket: &String,
    project: &String,
    prepend: &Option<String>,
    client: &Client,
) -> Result<(), Error> {
    let key_folder = match prepend {
        Some(pre) => {
            let mut name = pre.clone();
            name.push_str("/");
            name
        }
        None => String::from(""),
    };
    let modified_files = get_modified_files(project);
    for (key, filename) in modified_files {
        let mime = get_mime_type(&filename).expect("Could not get media type");
        let mut full_key = key_folder.clone();
        full_key.push_str(&key);
        // println!("{}", &full_key);
        // uncomment below to actually upload!
        upload_object(client, bucket, mime, &filename, &full_key).await?;
    }
    Ok(())
}

async fn deploy_single(
    bucket: &String,
    file: &String,
    prepend: &Option<String>,
    client: &Client,
) -> Result<(), Error> {
    // get key from file name
    // get MIME type
    // get prepend
    // upload asset
    println!("todo!");
    Ok(())
}

fn modified(project: &String) -> () {
    let modified_files = get_modified_files(project);
    println!("{} files modified recently: ", modified_files.len());
    for (key, _value) in modified_files {
        println!("   {}", key);
    }
}

#[derive(Parser)]
#[clap(name = "s3-deploy")]
#[clap(author = "olmigs <migs@mdguerrero.com>")]
#[clap(version = "1.0")]
#[clap(about = "Deploy your static site to AWS S3", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print objects in <BUCKET>
    Print {
        #[clap(short, long)]
        bucket: String,
    },
    /// Deploy recently modified files in <PROJECT> to <BUCKET>
    Yolo {
        #[clap(short, long)]
        bucket: String,
        #[clap(short, long)]
        project: String,
        #[clap(short, long)]
        subdirectory: Option<String>,
    },
    /// Print recently modified files (< 24 hrs) in <PROJECT>
    Modified {
        #[clap(short, long)]
        project: String,
    },
    /// Deploy <FILE> to <BUCKET>
    Upload {
        #[clap(short, long)]
        bucket: String,
        #[clap(short, long)]
        file: String,
        #[clap(short, long)]
        subdirectory: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    println!();

    match &cli.command {
        Commands::Print { bucket } => print(bucket, &client).await,
        Commands::Yolo {
            bucket,
            project,
            subdirectory,
        } => deploy(bucket, project, subdirectory, &client).await?,
        Commands::Modified { project } => modified(project),
        Commands::Upload {
            bucket,
            file,
            subdirectory,
        } => deploy_single(bucket, file, subdirectory, &client).await?,
    }
    Ok(())
}
