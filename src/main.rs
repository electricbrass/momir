use ansi_term::Color as TermColor;
use ansi_term::Style;
use clap::Parser;
use scryfall::card::Color as CardColor;
use scryfall::Card;
use scryfall::Error::ScryfallError;
use std::io;

#[derive(Parser, Debug)]
#[command(author, version, about = "Gives a random creature card based on the given mana value.", long_about = None)]
struct Args {
    #[arg(short = 'r', long = "replacement", help = "Allow cards to be repeated")]
    use_replacement: bool,
    #[arg(short = 'f', long = "funny", help = "Include funny (un-set) cards")]
    include_funny: bool,
    #[arg(
        short = 'g',
        long = "giant",
        help = "Use the additional Stonehewer Giant vanguard"
    )]
    use_giant: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let giant = args.use_giant;
    let mut creatures_seen = String::new();
    let mut equipment_seen = String::new();
    loop {
        println!("Please input a mana value (or q to quit).");
        let mut cmc = String::new();
        io::stdin()
            .read_line(&mut cmc)
            .expect("Failed to read line");

        if let Ok('q') = cmc.trim().parse::<char>() {
            break;
        };

        let cmc: u32 = match cmc.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please type a number!");
                continue;
            }
        };

        if giant {
            println!("--------------- Creature ---------------");
        }
        let query = format!("((-is:dfc AND t:creature) OR (t:/.*Creature.*\\//)) -is:digital -is:reversible cmc:{cmc} {creatures_seen} {}",
        if args.include_funny {""} else {"-is:funny"});

        get_card(&query, &mut creatures_seen, args.use_replacement).await;

        if giant {
            println!();
            println!("--------------- Equipment ---------------");
            let query = format!("((-is:dfc AND t:equipment) OR (t:/.*Equipment.*\\//)) -is:digital -is:reversible cmc<{cmc} {equipment_seen} {}",
            if args.include_funny {""} else {"-is:funny"});

            get_card(&query, &mut equipment_seen, args.use_replacement).await;
        }

        println!("----------------------------------------------");
    }
}

async fn get_card(query: &str, cards_seen: &mut String, use_replacement: bool) {
    let card: Card = match Card::search_random(query).await {
        Ok(c) => c,
        Err(ScryfallError(e))
            if e.details == "0 cards matched this search, a random card could not be returned." =>
        {
            println!(
                "{}",
                TermColor::Red.paint("No cards available with the requested mana value.")
            );
            return;
        }
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{}", e);
            }
            println!(
                "{}",
                TermColor::Red.paint("Something went wrong while fetching the card.")
            );
            println!("Please try again.");
            return;
        }
    };

    if !use_replacement {
        cards_seen.push_str(&format!(
            "-oracle_id:{} ",
            card.oracle_id.expect(
                "Only reversible cards do not have an oracle_id, and they have been excluded."
            )
        ));
    }

    let mut colors: Vec<TermColor> = card
        .color_identity
        .iter()
        .map(|c| match c {
            CardColor::Black => TermColor::Purple,
            CardColor::White => TermColor::Yellow,
            CardColor::Blue => TermColor::Blue,
            CardColor::Red => TermColor::Red,
            CardColor::Green => TermColor::Green,
            CardColor::Colorless => TermColor::Fixed(245),
        })
        .collect();
    if colors.is_empty() {
        colors.push(TermColor::Fixed(245))
    }
    print!("Card name:");
    for c in colors {
        print!(" {}", c.paint(&card.name));
    }
    println!();
    println!(
        "Scryfall link: {}",
        Style::new()
            .underline()
            .paint(format!("{}", &card.scryfall_uri))
    );
}
