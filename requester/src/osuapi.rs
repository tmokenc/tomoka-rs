// OsuApi v1, with change to v2 when it's stable.
use reqwest::Client as ReqwestClient;

pub trait OsuApiRequester {
    fn get_beatmap(&self, token: impl AsRef<str>) -> OsuBeatmap;
    fn get_user(&self, token: impl AsRef<str>) -> OsuUser;
    fn get_scores(&self, token: impl AsRef<str>) -> OsuScores;
    fn get_user_best(&self, token: impl AsRef<str>) -> OsuUserBest;
    fn get_user_recent(&self, token: impl AsRef<str>) -> OsuUserRecent;
    fn get_match(&self, token: impl AsRef<str>) -> OsuMatch;
    fn get_replay(&self, token: impl AsRef<str>) -> OsuReplay;
}

pub struct BeatmapParams {
    
}

pub struct OsuBeatmap {}

pub struct OsuUser {}

pub struct OsuScores {}

pub struct OsuUserBest {}

pub struct OsuUserRecent {}

pub struct OsuMatch {}

pub struct OsuReplay {}

impl OsuApiRequester for ReqwestClient {
    fn get_beatmap(&self, token: impl AsRef<str>) -> OsuBeatmap {
        
    }
    
    fn get_user(&self, token: impl AsRef<str>) -> OsuUser {
        
    }
    
    fn get_scores(&self, token: impl AsRef<str>) -> OsuScores {
        
    }
    
    fn get_user_best(&self, token: impl AsRef<str>) -> OsuUserBest {
        
    }
    
    fn get_user_recent(&self, token: impl AsRef<str>) -> OsuUserRecent {
        
    }
    
    fn get_match(&self, token: impl AsRef<str>) -> OsuMatch {
        
    }
    
    fn get_replay(&self, token: impl AsRef<str>) -> OsuReplay {
        
    }
    
}
