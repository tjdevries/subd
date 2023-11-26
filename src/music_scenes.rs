
pub struct NewVoiceScene {
    pub voice: &'static str,
    pub music: &'static str,
    pub playlist_folder: Option<&'static str>,
}

pub const VOICE_TO_MUSIC: &[(&str, NewVoiceScene)] = &[
        // ("!yoga", NewVoiceScene{ voice: "Thomas", music: "Yoga-BG-Music", playlist_folder: Some("Yoga")}),
        ("!yoga", NewVoiceScene{ voice: "god", music: "Yoga-BG-Music", playlist_folder: Some("Yoga")}),
        ("!drama", NewVoiceScene{ voice: "Ethan", music: "Dramatic-BG-Music", playlist_folder: Some("Drama")}),
        ("!greed", NewVoiceScene{ voice: "Michael", music: "Greed-BG-Music", playlist_folder: None }),
        ("!ken", NewVoiceScene{ voice: "James", music: "KenBurns-BG-Music", playlist_folder: Some("KenBurns")}),
        ("!hospital", NewVoiceScene{ voice: "Bella", music: "Hospital-BG-Music", playlist_folder: None }),
        ("!sigma", NewVoiceScene{ voice: "Ethan", music: "Sigma-BG-Music", playlist_folder: Some("Sigma")}),
        ("!news", NewVoiceScene{ voice: "Ethan", music: "News-1-BG-Music", playlist_folder: Some("News")}),
        ("!sexy", NewVoiceScene{ voice: "Charlotte", music: "Sexy-2-BG-Music", playlist_folder: Some("Sexy")}),
        ("!romcom", NewVoiceScene{ voice: "Fin", music: "Romcom-BG-Music", playlist_folder: None }),
        ("!chef", NewVoiceScene{ voice: "Giovanni", music: "Chef-BG-Music", playlist_folder: None }),
        ("!carti", NewVoiceScene{ voice: "Random", music: "Carti-BG-Music", playlist_folder: None }),
        ("!nerds", NewVoiceScene{ voice: "Fin", music: "Nerd-BG-Music", playlist_folder: None }),
        ("!earth", NewVoiceScene{ voice: "atten", music: "Planet-Earth-BG-Music-1", playlist_folder: Some("PlanetEarth")}),
];

