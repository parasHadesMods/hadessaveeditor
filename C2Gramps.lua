local room = CurrentRun.CurrentRoom
--room.StartRoomPresentationOnReload = true
room.ExitsUnlocked = false
room.Encounter.Completed = false
room.CheckObjectStatesOnStartRoom = true

room.ObjectStates[560571] = { 
    ForceRoomName = "I_Boss01",
    PreExitsUnlockedFunctionName = "SetPathValue",
    PreExitsUnlockedFunctionArgs = {
        TablePath = { "MapState", "OfferedExitDoors", 560571, "Room" },
        Key = "Flipped",
        Value = false

    }
}

GameState.LastRemembranceCompletedRunCount = nil
GameState.PlayedRandomRunIntroData.Intro_Hades = nil