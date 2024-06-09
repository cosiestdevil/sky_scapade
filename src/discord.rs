use bevy::prelude::*;
use discord_rich_presence::{activity::{self, Timestamps}, DiscordIpc, DiscordIpcClient};
pub struct DiscordPlugin;
#[derive(Resource, Deref, DerefMut)]
pub struct DiscordClient(Option<DiscordIpcClient>);

impl Plugin for DiscordPlugin {
    fn build(&self, app: &mut App) {
        let client = DiscordIpcClient::new(env!("DISCORD_CLIENT_ID")).unwrap();
        app.init_resource::<ActivityState>();
        app.insert_resource(DiscordClient(Some(client)));
        app.add_systems(Startup, startup_client);
        app.add_systems(FixedUpdate, check_activity_changed);
    }
}
fn startup_client(mut client: ResMut<DiscordClient>) {
    match &mut client.0{
        Some(discord) =>{
            if let Err(err) = discord.connect(){
                error!("{}",err);
                client.0 = None;
            }
        },
        None => {},
    }
    
}
fn check_activity_changed(activity: Res<ActivityState>, mut client: ResMut<DiscordClient>) {
    match &mut client.0{
        Some(client) => {
            if activity.is_changed() {
                let mut discord_activity: activity::Activity = activity::Activity::new();
                if let Some(state) = &activity.state {
                    discord_activity = discord_activity.state(state);
                }
                if let Some(details) = &activity.details {
                    discord_activity = discord_activity.details(details);
                }
                if let Some(start) = &activity.start{
                    discord_activity = discord_activity.timestamps(Timestamps::new().start(*start));
                }
                let res = client.set_activity(discord_activity);
        
                if let Err(why) = res {
                    error!("Failed to set presence: {}", why);
                }
            }
        },
        None => {},
    }
    
}
#[derive(Debug, Resource, Default, Clone)]
pub struct ActivityState {
    /// The player's current party status
    pub state: Option<String>,
    /// What the player is currently doing
    pub details: Option<String>,
    /// Start time of Activity
    pub start: Option<i64>,
}
