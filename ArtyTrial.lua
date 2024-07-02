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

-- any bounty other than this one will do; we can't override ourselves
-- because then the bounty can't complete out bounty (since this will)
-- return nil / be treated as false
BountyData["PackageBountyHealer"].UnlockGameStateRequirements = {
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
    }
}

function AddBoonRequirementsFunctionCall(bounty, name, args)
    table.insert(bounty.RewardStoreOverrides.RunProgress, {
        Name = "Boon",
        AllowDuplicates = true,
        GameStateRequirements =
        {
            {
                FunctionName = name,
                FunctionArgs = args
            }
        }
    })
end

local f_rooms = {
    "F_Opening01",
    "F_Opening02",
    "F_Opening03",
    "F_MiniBoss01",
    "F_MiniBoss02",
    "F_Combat01",
    "F_Combat02",
    "F_Combat03",
    "F_Combat04",
    "F_Combat05",
    "F_Combat06",
    "F_Combat07",
    "F_Combat08",
    "F_Combat09",
    "F_Combat10",
    "F_Combat11",
    "F_Combat12",
    "F_Combat13",
    "F_Combat14",
    "F_Combat15",
    "F_Combat16",
    "F_Combat17",
    "F_Combat18",
    "F_Combat19",
    "F_Shop01",
    "F_Reprieve01",
    "F_Story01",
    "Chaos_01",
    "Chaos_02",
    "Chaos_03",
    "Chaos_04",
    "Chaos_05",
    "Chaos_06",
}

for i, roomName in ipairs(f_rooms) do
    AddBoonRequirementsFunctionCall(
        bounty,
        "SetPathValue",
        {
            TablePath = { "RoomData", roomName },
            Key = "LeavePostPresentationEvents",
            Value = {
                {
                    FunctionName = "RandomSynchronize",
                    GameStateRequirements = {
                        {
                            PathTrue = { "CurrentRun", "ActiveBounty" },
                        }
                    }
                }
            }
        }
    )
end

-- AddBoonRequirementsFunctionCall(
--     bounty,
--     "SetPathValue",
--     {
--         TablePath = { "CurrentRun", "CurrentRoom" },
--         Key = 1,
--         Value = {
--             FunctionName = "RandomSynchronize",
--         }
--     }
-- )

bounty.ShrineUpgradesActive = {}