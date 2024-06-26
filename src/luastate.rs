use crate::luabins;
use anyhow::Result;
use rlua::{Lua, Value};

pub fn load(lua: &Lua, lua_state: &mut &[u8]) -> Result<()> {
    lua.context(|lua_ctx| -> Result<()> {
        let save_data = luabins::load(lua_state, lua_ctx)?;
        lua_ctx.globals().set("_saveData", save_data)?;
        // put save file data into globals
        lua_ctx.load(r#"
            for _,savedValues in pairs(_saveData) do
                for key, value in pairs(savedValues) do
                    if not SaveIgnores[key] then
                        _G[key] = value
                    end
                end
            end
            _saveData = nil
        "#).exec().map_err(anyhow::Error::new)
    })
}

pub fn save(lua: &Lua) -> Result<Vec<u8>> {
    let mut new_lua_state: Vec<u8> = Vec::new();
    lua.context(|lua_ctx| -> Result<()> {
        // read save file data from
        lua_ctx.load(r#"  
            _saveData = { [1] = {}}

            if GlobalSaveWhitelist ~= nil then
                for i, key in ipairs( GlobalSaveWhitelist ) do
                    local value = _G[key]
                    if value ~= nil then
                        local valueType = type(value)
                        if valueType ~= "function" and valueType ~= "userdata" and valueType ~= "thread" then
                            _saveData[1][key] = value
                        end
                    end
                end
            else
                for key, value in pairs( _G ) do
                    if value ~= nil and not SaveIgnores[key] then
                            local valueType = type(value)
                            if valueType ~= "function" and valueType ~= "userdata" and valueType ~= "thread" then
                                _saveData[1][key] = value
                            end
                    end
                end
            end

        "#).exec().map_err(anyhow::Error::new)?;
        let save_data: Vec<Value> = lua_ctx.globals().get("_saveData").map_err(anyhow::Error::new)?;
        luabins::save(&mut new_lua_state, save_data)
    })?;
    Ok(new_lua_state)
}

pub fn initialize_v17(lua: &Lua) -> Result<()> {
    lua.context(|lua_ctx| -> Result<()> {
        lua_ctx.load(r#"
            GlobalSaveWhitelist = {
                "GameState",
                "StoredGameState",
                "CurrentRun",
                "MapState",
                "AudioState",
                "CurrentHubRoom",
                "CodexStatus",
                "_worldTime",
                "_worldTimeUnmodified",
                "Revision",
                "NextSeeds",
            }
            SaveIgnores = {}
        "#).exec().map_err(anyhow::Error::new)
    })
}


pub fn initialize_v16(lua: &Lua) -> Result<()> {
    lua.context(|lua_ctx| -> Result<()> {
        lua_ctx.load(r#"
            SaveIgnores = {}
            for _,value in pairs( {
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
            } ) do
                SaveIgnores[value] = true
            end
        "#).exec().map_err(anyhow::Error::new)
    })
}