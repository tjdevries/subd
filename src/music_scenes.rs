
pub struct NewVoiceScene {
    pub voice: &'static str,
    pub music: &'static str,
    // scene: String,
}

pub const VOICE_TO_MUSIC: &[(&str, NewVoiceScene)] = &[
        ("!yoga", NewVoiceScene{ voice: "Thomas", music: "Yoga-BG-Music" }),
        ("!drama", NewVoiceScene{ voice: "Ethan", music: "Dramatic-BG-Music" }),
        ("!greed", NewVoiceScene{ voice: "Michael", music: "Greed-BG-Music" }),
        ("!ken", NewVoiceScene{ voice: "James", music: "KenBurns-BG-Music" }),
        ("!hospital", NewVoiceScene{ voice: "Bella", music: "Hospital-BG-Music" }),
        ("!sigma", NewVoiceScene{ voice: "Ethan", music: "Sigma-BG-Music" }),
        ("!news", NewVoiceScene{ voice: "Ethan", music: "News-1-BG-Music" }),
        ("!sexy", NewVoiceScene{ voice: "Charlotte", music: "Sexy-2-BG-Music" }),
        ("!romcom", NewVoiceScene{ voice: "Fin", music: "Romcom-BG-Music" }),
        ("!chef", NewVoiceScene{ voice: "Giovanni", music: "Chef-BG-Music" }),
        ("!carti", NewVoiceScene{ voice: "Random", music: "Carti-BG-Music" }),
        ("!nerds", NewVoiceScene{ voice: "Fin", music: "Nerd-BG-Music" }),
];

