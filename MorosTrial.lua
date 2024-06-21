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

unlock_bounty("PackageBountyHealer")

local bounty = BountyData.PackageBountyHealer

bounty.Text = "Trial of the Week #2"
bounty.WeaponKitName = "WeaponTorch"
bounty.WeaponUpgradeName = "TorchDetonateAspect"
bounty.KeepsakeName = "BlockDeathKeepsake"
bounty.Repeatable = true

bounty.StartingTraits = {
    { Name = "HeraWeaponBoon", Rarity = "Epic", },
    { Name = "AphroditeSpecialBoon", Rarity = "Epic", },
    { Name = "HeraManaBoon", Rarity = "Epic", },
}

bounty.RunOverrides =
{
    MaxGodsPerRun = 3,
    LootTypeHistory =
    {
        AphroditeUpgrade = 1,
        HeraUpgrade = 2,
    }
}

bounty.MetaUpgradeStateEquipped =
{
    "BonusRarity",
    "LastStand",
}

bounty.RewardStoreOverrides.RunProgress = {
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
}

bounty.ShrineUpgradesActive = {}

NextSeeds[1] = nil