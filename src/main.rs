use std::{
    fmt::{self, Formatter},
    fs::File,
};

use chrono::{Datelike, Months, NaiveDate};
use clap::Parser;

use csv;

use serde::{Deserialize, Deserializer};

enum Frequency {
    OneTime(Option<NaiveDate>),
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl Frequency {
    fn from_str(s: &str) -> Result<Self, &'static str> {
        let s = s.to_lowercase();
        let parts = s.split('(').collect::<Vec<_>>();
        match parts[0] {
            "1" | "once" | "onetime" => {
                let date = parts
                    .get(1)
                    .and_then(|s| s.strip_suffix(')'))
                    .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
                Ok(Frequency::OneTime(date))
            }
            "d" | "day" | "daily" => Ok(Frequency::Daily),
            "w" | "week" | "weekly" => Ok(Frequency::Weekly),
            "biweekly" => Ok(Frequency::BiWeekly),
            "m" | "month" | "monthly" => Ok(Frequency::Monthly),
            "quarter" | "quarterly" => Ok(Frequency::Quarterly),
            "y" | "year" | "yearly" => Ok(Frequency::Yearly),
            _ => Err("Invalid frequency"),
        }
    }
}

impl fmt::Debug for Frequency {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Frequency::OneTime(date) => {
                if let Some(date) = date {
                    write!(f, "OneTime({})", date.format("%Y-%m-%d"))
                } else {
                    write!(f, "OneTime")
                }
            }
            Frequency::Daily => write!(f, "Daily"),
            Frequency::Weekly => write!(f, "Weekly"),
            Frequency::BiWeekly => write!(f, "BiWeekly"),
            Frequency::Monthly => write!(f, "Monthly"),
            Frequency::Quarterly => write!(f, "Quarterly"),
            Frequency::Yearly => write!(f, "Yearly"),
        }
    }
}

impl<'de> Deserialize<'de> for Frequency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Frequency::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
enum Type {
    Bill,
    Income,
    Investment,
    Subscription,
    Other,
}

impl Type {
    fn from_str(s: &str) -> Result<Self, &'static str> {
        match s.to_lowercase().as_str() {
            "bill" => Ok(Type::Bill),
            "income" => Ok(Type::Income),
            "investment" => Ok(Type::Investment),
            "sub" | "subscription" => Ok(Type::Subscription),
            "other" => Ok(Type::Other),
            _ => Err("Invalid type"),
        }
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Type::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
struct CashEvent {
    name: String,
    usd: f64,
    frequency: Frequency,
    type_: Type,
    is_taxable: bool,
}

impl CashEvent {
    fn _new(
        name: String,
        usd: f64,
        frequency: Frequency,
        type_: Type,
        is_taxable: Option<bool>,
    ) -> CashEvent {
        CashEvent {
            name,
            usd,
            frequency,
            type_,
            is_taxable: is_taxable.unwrap_or(false),
        }
    }

    fn get_monthly_amount(&self, date: &NaiveDate, tax_rate: f64) -> f64 {
        let amount = if self.is_taxable {
            self.usd * (1.0 - tax_rate)
        } else {
            self.usd
        };

        let amount = match self.frequency {
            Frequency::OneTime(one_time_date) => {
                if let Some(one_time_date) = one_time_date {
                    if date.month() == one_time_date.month() && date.year() == one_time_date.year()
                    {
                        return amount;
                    }
                }
                return 0.0;
            }
            Frequency::Daily => amount * 30.0,
            Frequency::Weekly => amount * 4.5,
            Frequency::BiWeekly => amount * 2.25,
            Frequency::Monthly => amount,
            Frequency::Quarterly => amount / 3.0,
            Frequency::Yearly => amount / 12.0,
        };

        match self.type_ {
            Type::Income => amount,
            _ => -amount,
        }
    }
}

fn get_monthly_amount(cash_events: &Vec<CashEvent>, date: &NaiveDate, tax_rate: f64) -> f64 {
    cash_events
        .iter()
        .map(|cash_event| cash_event.get_monthly_amount(date, tax_rate))
        .sum()
}

fn get_first_day_of_months_between(start_date: &NaiveDate, end_date: &NaiveDate) -> Vec<NaiveDate> {
    if start_date > end_date {
        return vec![];
    }
    let mut dates = vec![];
    let mut date = start_date.clone().with_day(1).expect("Invalid date");
    while date <= *end_date {
        if date.day() == 1 {
            dates.push(date);
        }
        date = date
            .checked_add_signed(chrono::Duration::days(1))
            .expect("Invalid date");
    }
    dates
}

#[cfg(test)]
mod get_first_day_of_months_between {

    #[test]
    fn first_day_to_first_day() {
        use super::get_first_day_of_months_between;
        use chrono::NaiveDate;

        let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2020, 3, 1).unwrap();
        let dates = get_first_day_of_months_between(&start_date, &end_date);
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2020, 2, 1).unwrap());
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2020, 3, 1).unwrap());
    }

    #[test]
    fn middle_to_middle() {
        use super::get_first_day_of_months_between;
        use chrono::NaiveDate;

        let start_date = NaiveDate::from_ymd_opt(2020, 1, 20).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2020, 3, 20).unwrap();
        let dates = get_first_day_of_months_between(&start_date, &end_date);
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert_eq!(dates[1], NaiveDate::from_ymd_opt(2020, 2, 1).unwrap());
        assert_eq!(dates[2], NaiveDate::from_ymd_opt(2020, 3, 1).unwrap());
    }

    #[test]
    fn start_and_end_in_same_month() {
        use super::get_first_day_of_months_between;
        use chrono::NaiveDate;

        let start_date = NaiveDate::from_ymd_opt(2020, 1, 20).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2020, 1, 25).unwrap();
        let dates = get_first_day_of_months_between(&start_date, &end_date);
        assert_eq!(dates.len(), 1);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    }

    #[test]
    fn return_empty_if_start_after_end() {
        use super::get_first_day_of_months_between;
        use chrono::NaiveDate;

        let start_date = NaiveDate::from_ymd_opt(2020, 1, 20).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2020, 1, 15).unwrap();
        let dates = get_first_day_of_months_between(&start_date, &end_date);
        assert_eq!(dates.len(), 0);
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long, default_value = "data/cash_events.csv")]
    cash_events_file_path: String,

    #[arg(short, long, default_value = "12")]
    months: u32,

    #[arg(short, long, default_value = "0.169")]
    tax_rate: f64,
}

fn main() {
    let args: Args = Args::parse();

    let start_date = chrono::Local::now().naive_local().date();
    let end_date = start_date
        .checked_add_months(Months::new(args.months))
        .expect("Invalid date");

    let dates = get_first_day_of_months_between(&start_date, &end_date);

    let file = File::open(args.cash_events_file_path).expect("Unable to open file");
    let mut reader = csv::Reader::from_reader(file);
    let mut events: Vec<CashEvent> = vec![];
    for result in reader.deserialize() {
        let record: CashEvent = result.expect("Unable to parse record");
        if args.verbose {
            println!("{:?}", record);
        }
        events.push(record);
    }

    let mut sum = 0.0;
    for date in dates {
        let monthly_amount = get_monthly_amount(&events, &date, args.tax_rate);
        sum += monthly_amount;
        println!(
            "{}:\t{:10.2}\t==>\t{sum:10.2}",
            date.format("%Y-%m"),
            monthly_amount,
        );
    }
}
