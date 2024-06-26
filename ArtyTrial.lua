function starts_with(haystack, needle)
    return type(haystack) == "string" and haystack:sub(0, needle:len()) == needle
end

function lock_bounty(name)
    BountyData[name].UnlockGameStateRequirements.Skip = true
    BountyData[name].UnlockGameStateRequirements.Force = nil
end

function unlock_bounty(name)
    BountyData[name].UnlockGameStateRequirements.Skip = nil
    BountyData[name].UnlockGameStateRequirements.Force = true   
end

for name,data in pairs(BountyData) do
    if starts_with(name, "PackageBounty") then
        lock_bounty(name)
    end
end

for name,data in pairs(GameState.BountiesCompleted) do
    if starts_with(name, "PackageBounty") then
        GameState.BountiesCompleted[name] = nil
    end
end

unlock_bounty("PackageBountyHellChop")

local bounty = BountyData.PackageBountyHellChop

bounty.Text = "Trial of the Week #3"
bounty.WeaponKitName = "WeaponDagger"
bounty.WeaponUpgradeName = "DaggerBlockAspect"
bounty.KeepsakeName = "LowHealthCritKeepsake"
bounty.RemoveFamiliar = true
bounty.Repeatable = true

bounty.StartingTraits ={
    { Name = "AphroditeWeaponBoon", Rarity = "Epic", },
    { Name = "AphroditeManaBoon", Rarity = "Epic", },
    { Name = "WeakPotencyBoon", Rarity = "Epic", },
    { Name = "HighHealthCritBoon", Rarity = "Epic", },
    { Name = "FocusDamageShaveBoon", Rarity = "Heroic", },
}

bounty.ForcedRewards = nil

bounty.RunOverrides =
{
    MaxGodsPerRun = 2,
    LootTypeHistory =
    {
        AphroditeUpgrade = 4,
        PoseidonUpgrade = 1,
    },
}

bounty.MetaUpgradeStateEquipped = {
    "LowHealthBonus",
    "ChanneledBlock",
}

bounty.CompleteGameStateRequirements = {
    { 
        FunctionName = "SetPathValue",
        FunctionArgs = {
            TablePath = { "NextSeeds" },
            Key = 1,
            Value = NextSeeds[1]
        }
    }
}

bounty.RewardStoreOverrides = {
    RunProgress = {
        {
            Name = "MaxManaDrop",
            GameStateRequirements =
            {
                -- None
            }
        },
        {
            Name = "RoomMoneyDrop",
            GameStateRequirements =
            {
                -- None
            },
        },
        {
            Name = "StackUpgrade",
            GameStateRequirements =
            {
                NamedRequirements = { "StackUpgradeLegal", },
            }
        },
        {
            Name = "WeaponUpgrade",
            GameStateRequirements =
            {
                -- None
            }
        },
        {
            Name = "Boon",
            AllowDuplicates = true,
            GameStateRequirements =
            {
                -- None
            },
        },
        {
            Name = "Boon",
            AllowDuplicates = true,
            GameStateRequirements =
            {
                -- None
            },
        },
        {
            Name = "Boon",
            AllowDuplicates = true,
            GameStateRequirements =
            {
                -- None
            },
        },
        {
            Name = "Boon",
            AllowDuplicates = true,
            GameStateRequirements =
            {
                -- None
            },
        },
        {
            Name = "Boon",
            AllowDuplicates = true,
            GameStateRequirements =
            {
                {
                    FunctionName = "SetPathValue",
                    FunctionArgs = {
                        TablePath = { "CurrentRun", "CurrentRoom" },
                        Key = "LeaveEvents",
                        Value = {
                            {
                                FunctionName = "RunEventsGeneric",
                            }
                        }
                    }
                }
            }
        },
        {
            Name = "Boon",
            AllowDuplicates = true,
            GameStateRequirements =
            {
                {
                    FunctionName = "SetPathValue",
                    FunctionArgs = {
                        TablePath = { "CurrentRun", "CurrentRoom" },
                        Key = 1,
                        Value = {
                            FunctionName = "RandomSynchronize",
                        }
                    }
                }
            }
        },
        -- {
        --     Name = "Boon",
        --     AllowDuplicates = true,
        --     GameStateRequirements =
        --     {
        --         {
        --             FunctionName = "SetPathValue",
        --             FunctionArgs = {
        --                 TablePath = { "HubRoomData", "Flashback_Hub_Main" },
        --                 Key = "OnDeathLoadRequirements",
        --                 Value = {
        --                     {
        --                         {
        --                             FunctionName = "SetPathValue",
        --                             FunctionArgs = {
        --                                 TablePath = { "NextSeeds" },
        --                                 Key = 1,
        --                                 Value = NextSeeds[1]
        --                             }
        --                         }
        --                     },
        --                     {
        --                         {
        --                             FunctionName = "SetPathValue",
        --                             FunctionArgs = {
        --                                 TablePath = { "CurrentRun" },
        --                                 Key = 1,
        --                                 Value  = {
        --                                     FunctionName = "RandomSynchronize"
        --                                 }
        --                             }
        --                         }
        --                     },
        --                     {
        --                         {
        --                             FunctionName = "RunEventsGeneric"
        --                         }
        --                     }
        --                 }
        --             }
        --         }
        --     }
        -- },
    }
}

bounty.ShrineUpgradesActive = {}