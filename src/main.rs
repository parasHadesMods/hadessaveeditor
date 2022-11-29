mod luabins;
mod read;
mod hadesfile;
mod write;

use anyhow::Result;
use clap::{arg, Command};
use rlua::{Function, Lua, MultiValue};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use hadesfile::UncompressedSize;
use std::fs;
use std::path::{Path, PathBuf};
use lz4;

fn cli() -> Command {
    Command::new("hadessaveeditor")
        .about("A save file editor for hades")
        .arg(arg!(file: [FILE]).value_parser(clap::value_parser!(PathBuf)))
        .arg_required_else_help(true)
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    let path: &PathBuf = matches.get_one("file").expect("required");

    let lua = unsafe {
        Lua::new_with_debug()
    };

    let file = read_file(path)?;
    let mut savedata = hadesfile::read(&mut file.as_slice())?;
    let lua_state = lz4::block::decompress(
        &savedata.lua_state_lz4.as_slice(), 
        Some(hadesfile::HadesSaveV16::UNCOMPRESSED_SIZE))?;


    lua.context(|lua_ctx| -> Result<()> {
        let save_data = luabins::load(&mut lua_state.as_slice(), lua_ctx)?;
        lua_ctx.globals().set("_saveData", save_data)?;
        // put save file data into globals
        lua_ctx.load(r#"
function ToLookup( table )
        local lookup = {}
        for key,value in pairs( table ) do
                lookup[value] = true
        end
        return lookup
end

SaveIgnores = ToLookup({
    "debug",
    "package",
    "luanet",
    "_G",
    "os",
    "coroutine",
    "table",
    "_currentLine",
    "_currentFileName",
    "_currentStackLevel",
    "lastGoodThreadInfo",
    "string",
    "math",
    "io",
    "_VERSION",
    "Pickle",
    "_threadStack",
    "_threads",
    "_workingThreads",
    "_tagsToKill",
    "_events",
    "bit32",
    "_eventListeners",
    "_eventTimeoutRecord",
    "error",
    "pcall",
    "rawequal",
    "tostring",
    "getmetatable",
    "setmetatable",
    "rawlen",
    "rawget",
    "loadfile",
    "next",
    "rawset",
    "tonumber",
    "xpcall",
    "print",
    "type",
    "select",
    "dofile",
    "setmetatable",
    "collectgarbage",
    "assert",
    "pairs",
    "ipairs",
    "load",
    "resume",
    "char",
    "newFunctionCall",
    "NewStack",
    "SetCurrentLine",
    "TO_SAVE",
    "luabins",
    "utf8",
    "DebugFunctionIgnores",
    "MainFileFunctions",
    "global_triggerArgs",
    "SaveIgnores",
    "RoomSaveBlacklist",
    "RoomSaveWhitelist",
    "EncounterSaveBlacklist",
    "RunSaveWhitelist",
    "_saveData",
    "HotLoadInfo",
    "ForceEvent",
    "BlockHeroDeath",
    "verboseLogging",

    -- Control tables - Only used at run-time, e.g. for deferring presentation functions
    "CombatPresentationCaps",
    "CombatPresentationDeferredHealthBars",
    "UIScriptsDeferred",
    "DeferredPlayVoiceLines",

    -- Game Data - Objects which should be considered const by the rest of the scripts
    "GameData",
    "ConstantsData",
    "GameStateFlagData",
    "TextFormats",
    "Color",
    "EnemyData",
    "UnitSetData",
    "PresetEventArgs",
    "EncounterData",
    "RoomData",
    "RoomSetData",
    "BiomeMap",
    "HeroData",
    "GlobalModifiers",
    "LootData",
    "RewardStoreData",
    "MarketData",
    "BrokerData",
    "BrokerScreenData",
    "MetaUpgradeData",
    "TraitMultiplierData",
    "TraitData",
    "WeaponData",
    "ProjectileData",
    "EffectData",
    "SellTraitData",
    "StoreData",
    "GhostData",
    "WeaponUpgradeData",
    "BoonInfoScreenData",
    "FishingData",
    "EnemyUpgradeData",
    "ConsumableData",
    "ResourceData",
    "ConditionalItemData",
    "QuestData",
    "QuestOrderData",
    "ObstacleData",
    "CreditsData",
    "CreditsFormat",
    "CreditSpacing",
    "GlobalVoiceLines",
    "HeroVoiceLines",
    "WeaponSets",
    "UnitSets",
    "EnemySets",
    "Codex",
    "DeathLoopData",
    "BiomeMapGraphics",
    "ObjectiveData",
    "ObjectiveSetData",
    "ScreenData",
    "CodexUI",
    "CombatUI",
    "ShopUI",
    "LevelUpUI",
    "HealthUI",
    "SuperUI",
    "TraitUI",
    "ConsumableUI",
    "AmmoUI",
    "GunUI",
    "MoneyUI",
    "UIData",
    "ResourceData",
    "ResourceOrderData",
    "PlayerAIPersonaData",

    "IconData",

    "MusicTrackData",
    "MusicPlayerTrackData",
    "MusicPlayerTrackOrderData",
    "RoomStartMusicEvents",
    "CombatOverMusicEvents",
    "AmbienceTracks",

    "KeywordList",
    "Keywords",

    "HeroPhysicalWeapons",
    "WaveDifficultyPatterns",
    "TimerBlockCombatExcludes",
    "EncounterSets",
    "CodexOrdering",
    "CodexUnlockTypes",
    "ShowingCodexUpdateAnimation",
    "Icons",
    "IconTooltips",
    "FormatContainerIds",
    "RunIntroData",
    "GameOutroData",
    "EpilogueData",
    "MetaUpgradeLockOrder",
    "MetaUpgradeOrder",
    "ShrineUpgradeOrder",
    "ShrineClearData",
    "BiomeTimeLimits",
    "RerollCosts",
    "ScreenAnchors",
    "ScreenPresentationData",
    "EnemyHealthDisplayAnchors",
    "AssistUpgradeData",
    "GiftData",
    "GiftIconData",
    "GiftOrdering",
    "GiftOrderingReverseLookup",
    "BaseWaveOverrideValues",
    "ElysiumWaveOverrideValues",
    "IntroWaveOverrideValues",
    "MaterialDefaults",
    "BiomeList",
    "StatusAnimations",
    "DamageRecord",
    "SpawnRecord",
    "HealthRecord",
    "LifeOnKillRecord",
    -- Temp References - pointers are broken into serpate objects on save/load
    "AnchorId",
    "AdditionalDataAnchorId",
    "TraitInfoCardId",
    "AdvancedTooltipIcon",
    "TooltipData",
    "IdsTable",
    "IdsByTypeTable",
    "MusicId",
    "SecretMusicId",
    "SecretMusicName",
    "StoppingMusicId",
    "AmbientMusicId",
    "AmbienceId",
    "AmbienceName",
    "MapState",
    "SessionState",
    "AudioState",
    "ActiveEnemies",
    "RequiredKillEnemies",
    "ActiveObstacles",
    "ActivatedObjects",
    "LootObjects",
    "SurroundEnemiesAttacking",
    "LastEnemyKilled",
    "CurrentLootData",
    "CurrentMetaUpgradeName",
    "TempTextData",
    "LocalizationData",
    "TextLinesCache",
    "GlobalCooldowns",
    "GlobalCounts",
    "OfferedExitDoors",
    "SessionAchivementUnlocks",
})

for _,savedValues in pairs(_saveData) do
    for key, value in pairs(savedValues) do
        if not SaveIgnores[key] then
            _G[key] = value
        end
    end
end
_saveData = nil
        "#).exec().map_err(anyhow::Error::new)
    })?;

    repl(&lua)?;

    lua.context(|lua_ctx| -> Result<()> {
        // read save file data from
        lua_ctx.load(r#"  
        _saveData = { [1] = {}}
for key, value in pairs( _G ) do
        if value ~= nil and not SaveIgnores[key] then
                local valueType = type(value)
                if valueType ~= "function" and valueType ~= "userdata" and valueType ~= "thread" then
                    _saveData[1][key] = value
                end
        end
end
        "#).exec().map_err(anyhow::Error::new)?;
        let save_data = lua_ctx.globals().get("_saveData")?;
        let mut lua_state: Vec<u8> = Vec::new();
        luabins::save(&mut lua_state, save_data)?;
        savedata.lua_state_lz4 = lz4::block::compress(&lua_state, None, false)?;
        Ok(())
    })?;

    let outfile = hadesfile::write(&savedata)?;
    write_file(path, outfile)?;
    Ok(())
}

fn repl(lua: &Lua) -> Result<()> {
    let mut editor = Editor::<()>::new()?;
    loop {
        let readline = editor.readline(">> ");

        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_str());
                lua.context(|lua_ctx| -> Result<()> {
                    let result: MultiValue = lua_ctx.load(&line).eval()?;
                    let print: Function = lua_ctx.globals().get("print")?;
                    print.call(result)?;
                    Ok(())
                })?
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break
            },
            Err(err) => {
                println!("Unknown error: {:?}", err);
                break
            }
        }
    }
    Ok(())
}


const BYTE_ORDER_MARK: &[u8] = "\u{feff}".as_bytes();
fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
  let file = fs::read(path)?;
  if file.starts_with(BYTE_ORDER_MARK) {
     Ok(file[3..].to_vec())
  } else {
     Ok(file.to_vec())
  }
}

fn write_file<P: AsRef<Path>>(path: P, data: Vec<u8>) -> Result<()> {
    fs::write(path, data).map_err(anyhow::Error::new)
}
