use rand::seq::IteratorRandom;
use serde::Serialize;
use serde_json::{self};
use serenity::{
    self,
    client::Client,
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{channel::Message, id::UserId},
    prelude::{Context, EventHandler, *},
};
use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::Path,
};

struct ReccList;
impl TypeMapKey for ReccList {
    type Value = Vec<String>;
}

const TOKEN: &str = "oO87cTQKApWWOiGJbEytPQlfcYU.QoORkX.zMDMwIDOzQjMzMDNxkTN2MTN";
const RECC_FILE: &str = "./reccs.json";

#[group]
#[commands(recc, unrecc, show_reccs, pick_recc)]
struct General;

struct Handler;
impl EventHandler for Handler {}

fn dump<T, P>(obj: T, path: P)
where
    T: Serialize,
    P: AsRef<Path> + std::fmt::Debug,
{
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap_or_else(|err| panic!("Error opening file {:?} for writing: {:?}", &path, err));
    let mut writer = BufWriter::new(&file);
    serde_json::to_writer_pretty(&mut writer, &obj)
        .unwrap_or_else(|err| panic!("Error writing to file {:?}: {:?}", &path, err));
    writer.flush().expect("Unable to flush writer");
}

fn main() {
    let token: String = TOKEN.chars().rev().collect();
    let mut client = Client::new(token, Handler).expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!")) // set the bot's prefix to '!'
            .group(&GENERAL_GROUP)
            .help(&MY_HELP),
    );

    let path = std::path::Path::new(RECC_FILE);
    let recc_list = if path.is_file() {
        let file = File::open(RECC_FILE)
            .unwrap_or_else(|_| panic!("Error reading JSON file {}", RECC_FILE));
        let reader = BufReader::new(file);
        let list: Vec<String> = serde_json::from_reader(reader).expect("Expected JSON array");
        list
    } else {
        Vec::new()
    };

    {
        let mut data = client.data.write();
        data.insert::<ReccList>(recc_list.iter().cloned().collect());
    }

    if let Err(why) = client.start() {
        eprintln!("An error occured while starting the client: {}", why);
    }
}

fn sanatize_movie_name(mut args: Args) -> String {
    args.trimmed()
        .iter::<String>()
        .map(|x| x.unwrap())
        .collect::<Vec<String>>()
        .join(" ")
}

#[command]
fn recc(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let movie_name = sanatize_movie_name(args);
    let lower_movie_name = movie_name.to_lowercase();

    let mut data = ctx.data.write();
    let reccs = data
        .get_mut::<ReccList>()
        .expect("Expected ReccList in ShareMap");
    if reccs.contains(&lower_movie_name) {
        msg.channel_id
            .say(
                &ctx,
                format!(
                    "*{}* is already in the reccomendation list (`!show_reccs` to view already \
                     reccomended items)",
                    movie_name
                ),
            )
            .unwrap();
    } else {
        msg.channel_id
            .say(
                &ctx,
                format!("*{}* reccomended by {}", movie_name, msg.author),
            )
            .unwrap();
        reccs.push(movie_name);
        dump(reccs, RECC_FILE);
    }

    Ok(())
}

#[command]
fn unrecc(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let movie_name = sanatize_movie_name(args);
    let lower_movie_name = movie_name.to_lowercase();

    let mut data = ctx.data.write();
    let reccs = data
        .get_mut::<ReccList>()
        .expect("Expected ReccList in ShareMap");
    if reccs.contains(&lower_movie_name) {
        msg.channel_id
            .say(
                &ctx,
                format!("*{}* unreccomended by {}", movie_name, msg.author),
            )
            .unwrap();
        reccs.retain(|x| *x != lower_movie_name);
        dbg!(&reccs);
        dump(reccs, RECC_FILE);
    } else {
        msg.channel_id
            .say(
                &ctx,
                format!(
                    "*{}* is not in the reccomendation list (`!show_reccs` to view already \
                     reccomended items)",
                    movie_name
                ),
            )
            .unwrap();
    }

    Ok(())
}

#[command]
fn show_reccs(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read();
    let reccs = data
        .get::<ReccList>()
        .expect("Expected ReccList in ShareMap");
    let formatted_reccs = reccs.iter().map(|s| format!("- *{}*\n", s));
    let recc_msg = format!(
        "Reccomendations:\n{}",
        formatted_reccs.clone().collect::<String>()
    );
    // max length of a single message is 2000 codepoints/4000 bytes
    if recc_msg.len() <= 2000 {
        msg.channel_id.say(&ctx, recc_msg).unwrap();
    } else {
        // send the list in mesages of 1900 codepoints
        let mut recc_msg = String::new();
        for recc in formatted_reccs {
            recc_msg.push_str(&recc);
            if recc_msg.len() >= 1900 {
                msg.channel_id.say(&ctx, recc_msg).unwrap();
                recc_msg = String::new();
            }
        }
        if !recc_msg.is_empty() {
            msg.channel_id.say(&ctx, recc_msg).unwrap();
        }
    }

    Ok(())
}

#[command]
fn pick_recc(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut rng = rand::thread_rng();
    let data = ctx.data.read();
    let reccs = data
        .get::<ReccList>()
        .expect("Expected ReccList in ShareMap");
    let picked_recc = reccs.iter().choose(&mut rng);

    match picked_recc {
        Some(picked_recc) => msg
            .channel_id
            .say(&ctx, format!("Picked reccomendation: *{}*", picked_recc))
            .unwrap(),
        None => msg
            .channel_id
            .say(&ctx, "There are no reccomendations in the list")
            .unwrap(),
    };

    Ok(())
}

#[help]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}
