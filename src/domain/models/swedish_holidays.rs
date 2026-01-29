use chrono::{Datelike, NaiveDate, Weekday};

/// Represents the type of Swedish holiday
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HolidayType {
    /// Official public holiday (officiell helgdag)
    Public,
    /// De facto holiday - full day (de facto helgdag)
    DeFacto,
    /// De facto holiday - half day (de facto halvdag)
    DeFactoHalf,
}

/// Information about a Swedish holiday
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Holiday {
    pub(crate) date: NaiveDate,
    pub(crate) holiday_type: HolidayType,
    pub(crate) name: &'static str,
}

/// Checks if a date is a Swedish holiday and returns information about it
pub(crate) fn is_holiday(date: NaiveDate) -> Option<Holiday> {
    let year = date.year();
    let month = date.month();
    let day = date.day();

    // Fixed date holidays
    match (month, day) {
        // New Year's Day (Nyårsdagen) - Public holiday
        (1, 1) => Some(Holiday {
            date,
            holiday_type: HolidayType::Public,
            name: "Nyårsdagen",
        }),
        // Twelfth Night (Trettondagsafton) - De facto half holiday
        (1, 5) => Some(Holiday {
            date,
            holiday_type: HolidayType::DeFactoHalf,
            name: "Trettondagsafton",
        }),
        // Epiphany (Trettondagen) - Public holiday
        (1, 6) => Some(Holiday {
            date,
            holiday_type: HolidayType::Public,
            name: "Trettondagen",
        }),
        // Valborg (Valborgsmässoafton) - De facto half holiday
        (4, 30) => Some(Holiday {
            date,
            holiday_type: HolidayType::DeFactoHalf,
            name: "Valborgsmässoafton",
        }),
        // May Day (Första maj) - Public holiday
        (5, 1) => Some(Holiday {
            date,
            holiday_type: HolidayType::Public,
            name: "Första maj",
        }),
        // National Day (Sveriges nationaldag) - Public holiday
        (6, 6) => Some(Holiday {
            date,
            holiday_type: HolidayType::Public,
            name: "Sveriges nationaldag",
        }),
        // Christmas Eve (Julafton) - De facto holiday
        (12, 24) => Some(Holiday {
            date,
            holiday_type: HolidayType::DeFacto,
            name: "Julafton",
        }),
        // Christmas Day (Juldagen) - Public holiday
        (12, 25) => Some(Holiday {
            date,
            holiday_type: HolidayType::Public,
            name: "Juldagen",
        }),
        // Boxing Day (Annandag jul) - Public holiday
        (12, 26) => Some(Holiday {
            date,
            holiday_type: HolidayType::Public,
            name: "Annandag jul",
        }),
        // New Year's Eve (Nyårsafton) - De facto holiday
        (12, 31) => Some(Holiday {
            date,
            holiday_type: HolidayType::DeFacto,
            name: "Nyårsafton",
        }),
        _ => None,
    }
    .or_else(|| {
        // Easter-based holidays
        let easter = easter_date(year);
        let easter_offset = date.signed_duration_since(easter).num_days();

        match easter_offset {
            // Maundy Thursday (Skärtorsdagen) - De facto half holiday
            -3 => Some(Holiday {
                date,
                holiday_type: HolidayType::DeFactoHalf,
                name: "Skärtorsdagen",
            }),
            // Good Friday (Långfredagen) - Public holiday
            -2 => Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Långfredagen",
            }),
            // Holy Saturday (Påskafton) - De facto half holiday
            -1 => Some(Holiday {
                date,
                holiday_type: HolidayType::DeFactoHalf,
                name: "Påskafton",
            }),
            // Easter Sunday (Påskdagen) - Public holiday
            0 => Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Påskdagen",
            }),
            // Easter Monday (Annandag påsk) - Public holiday
            1 => Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Annandag påsk",
            }),
            // Ascension Eve (Kristi himmelfärdsdag) - De facto half holiday
            // Easter + 5*7 + 3 = Easter + 38
            38 => Some(Holiday {
                date,
                holiday_type: HolidayType::DeFactoHalf,
                name: "Kristi himmelfärdsdag",
            }),
            // Ascension Day (Kristi himmelfärdsdag) - Public holiday
            // Easter + 5*7 + 4 = Easter + 39
            39 => Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Kristi himmelfärdsdag",
            }),
            // Whit Sunday (Pingstdagen) - Public holiday
            // Easter + 7*7 = Easter + 49
            49 => Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Pingstdagen",
            }),
            // Whit Monday (Annandag pingst) - Public holiday
            // Easter + 7*7 + 1 = Easter + 50
            50 => Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Annandag pingst",
            }),
            _ => None,
        }
    })
    .or_else(|| {
        // Midsummer Eve (Midsommarafton) - Friday between June 20-26 - De facto holiday
        if date >= NaiveDate::from_ymd_opt(year, 6, 20).unwrap()
            && date <= NaiveDate::from_ymd_opt(year, 6, 26).unwrap()
            && date.weekday() == Weekday::Fri
        {
            Some(Holiday {
                date,
                holiday_type: HolidayType::DeFacto,
                name: "Midsommarafton",
            })
        } else if date >= NaiveDate::from_ymd_opt(year, 6, 20).unwrap()
            && date <= NaiveDate::from_ymd_opt(year, 6, 26).unwrap()
            && date.weekday() == Weekday::Sat
        {
            // Midsummer Day (Midsommardagen) - Saturday between June 20-26 - Public holiday
            Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Midsommardagen",
            })
        } else {
            None
        }
    })
    .or_else(|| {
        // All Saints' Eve (Allhelgonaafton) - Friday between Oct 31 and Nov 6 - De facto half holiday
        if date >= NaiveDate::from_ymd_opt(year, 10, 31).unwrap()
            && date <= NaiveDate::from_ymd_opt(year, 11, 6).unwrap()
            && date.weekday() == Weekday::Fri
        {
            Some(Holiday {
                date,
                holiday_type: HolidayType::DeFactoHalf,
                name: "Allhelgonaafton",
            })
        } else if date >= NaiveDate::from_ymd_opt(year, 10, 31).unwrap()
            && date <= NaiveDate::from_ymd_opt(year, 11, 6).unwrap()
            && date.weekday() == Weekday::Sat
        {
            // All Saints' Day (Alla helgons dag) - Saturday between Oct 31 and Nov 6 - Public holiday
            Some(Holiday {
                date,
                holiday_type: HolidayType::Public,
                name: "Alla helgons dag",
            })
        } else {
            None
        }
    })
}

/// Calculates the date of Easter Sunday for a given year using Donald Knuth's algorithm
/// See http://sv.wikipedia.org/wiki/P%C3%A5skdagen#Algoritm_f.C3.B6r_p.C3.A5skdagen
fn easter_date(year: i32) -> NaiveDate {
    let g = (year % 19) + 1;
    let c = (year / 100) + 1;
    let x = (3 * c) / 4 - 12;
    let z = (8 * c + 5) / 25 - 5;
    let d = (5 * year) / 4 - x - 10;
    let mut e = (11 * g + 20 + z - x) % 30;

    if e == 24 || (e == 25 && g > 11) {
        e += 1;
    }

    let mut n = 44 - e;
    if n < 21 {
        n += 30;
    }
    n += 7 - ((d + n) % 7);

    let month = 3 + n / 31;
    let day = n % 31;
    
    // Handle case where n % 31 == 0 (shouldn't happen in practice, but be safe)
    let (month, day) = if day == 0 {
        (month - 1, 31)
    } else {
        (month, day)
    };

    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .expect("Easter date calculation should always produce a valid date")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_holidays() {
        // New Year's Day
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Nyårsdagen");

        // Twelfth Night (de facto half)
        let date = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFactoHalf);
        assert_eq!(holiday.name, "Trettondagsafton");

        // Epiphany
        let date = NaiveDate::from_ymd_opt(2024, 1, 6).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Trettondagen");

        // May Day
        let date = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Första maj");

        // Christmas Eve (de facto)
        let date = NaiveDate::from_ymd_opt(2024, 12, 24).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFacto);
        assert_eq!(holiday.name, "Julafton");

        // Christmas Day
        let date = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Juldagen");

        // Boxing Day
        let date = NaiveDate::from_ymd_opt(2024, 12, 26).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Annandag jul");

        // New Year's Eve (de facto)
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFacto);
        assert_eq!(holiday.name, "Nyårsafton");
    }

    #[test]
    fn test_easter_based_holidays_2024() {
        // Easter 2024 is March 31
        
        // Maundy Thursday (March 28, 2024) - de facto half
        let date = NaiveDate::from_ymd_opt(2024, 3, 28).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFactoHalf);
        assert_eq!(holiday.name, "Skärtorsdagen");

        // Good Friday (March 29, 2024)
        let date = NaiveDate::from_ymd_opt(2024, 3, 29).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Långfredagen");

        // Holy Saturday (March 30, 2024) - de facto half
        let date = NaiveDate::from_ymd_opt(2024, 3, 30).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFactoHalf);
        assert_eq!(holiday.name, "Påskafton");

        // Easter Sunday (March 31, 2024)
        let date = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Påskdagen");

        // Easter Monday (April 1, 2024)
        let date = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Annandag påsk");

        // Ascension Eve (May 8, 2024) - de facto half
        let date = NaiveDate::from_ymd_opt(2024, 5, 8).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFactoHalf);
        assert_eq!(holiday.name, "Kristi himmelfärdsdag");

        // Ascension Day (May 9, 2024)
        let date = NaiveDate::from_ymd_opt(2024, 5, 9).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Kristi himmelfärdsdag");

        // Whit Sunday (May 19, 2024)
        let date = NaiveDate::from_ymd_opt(2024, 5, 19).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Pingstdagen");

        // Whit Monday (May 20, 2024)
        let date = NaiveDate::from_ymd_opt(2024, 5, 20).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Annandag pingst");
    }

    #[test]
    fn test_midsummer_2024() {
        // Midsummer Eve 2024 should be June 21 (Friday between June 20-26)
        let date = NaiveDate::from_ymd_opt(2024, 6, 21).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFacto);
        assert_eq!(holiday.name, "Midsommarafton");

        // Midsummer Day 2024 should be June 22 (Saturday between June 20-26)
        let date = NaiveDate::from_ymd_opt(2024, 6, 22).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Midsommardagen");
    }

    #[test]
    fn test_all_saints_2024() {
        // All Saints' Eve 2024 should be November 1 (Friday between Oct 31 and Nov 6) - de facto half
        let date = NaiveDate::from_ymd_opt(2024, 11, 1).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFactoHalf);
        assert_eq!(holiday.name, "Allhelgonaafton");

        // All Saints' Day 2024 should be November 2 (Saturday between Oct 31 and Nov 6)
        let date = NaiveDate::from_ymd_opt(2024, 11, 2).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Alla helgons dag");
    }

    #[test]
    fn test_valborg_and_national_day() {
        // Valborg (April 30) - de facto half
        let date = NaiveDate::from_ymd_opt(2024, 4, 30).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::DeFactoHalf);
        assert_eq!(holiday.name, "Valborgsmässoafton");

        // National Day (June 6)
        let date = NaiveDate::from_ymd_opt(2024, 6, 6).unwrap();
        let holiday = is_holiday(date).unwrap();
        assert_eq!(holiday.holiday_type, HolidayType::Public);
        assert_eq!(holiday.name, "Sveriges nationaldag");
    }

    #[test]
    fn test_non_holiday() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        assert!(is_holiday(date).is_none());
    }

    #[test]
    fn test_easter_calculation() {
        // Verify Easter dates for a few years
        assert_eq!(easter_date(2024), NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());
        assert_eq!(easter_date(2023), NaiveDate::from_ymd_opt(2023, 4, 9).unwrap());
        assert_eq!(easter_date(2025), NaiveDate::from_ymd_opt(2025, 4, 20).unwrap());
    }
}
