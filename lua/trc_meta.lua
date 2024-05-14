---@meta

---@class T2SDataPacket
---@field Batch? T2SPacket[]
---@field SetupInfo? string
---@field Moved? {direction:MoveDir}
---@field SetMaxFuel? integer
---@field SetPos? {x:integer,y:integer,z:integer}
---@field SetOrientation? orienation
---@field WorldUpdate? string
---@field InventoryUpdate? {}
---@field NameUpdate? string
---@field FuelUpdate? integer
---@field Blocks? {up: Maybe<string>, down: Maybe<string>, front: Maybe<string>}
---@field Executables? string[]
---@field StdOut? string

---@alias T2SPacket T2SDataPacket
---| "Ping"

---@class S2TDataPacket
---@field RunLuaCode? string
---@field StdIn? string

---@alias S2TPacket S2TDataPacket | "GetSetupInfo" | "GetExecutables"

---@alias TrcExtensions "trc_position_tracking" | "trc_pathfinding"

---Follows SemVer, if major = 0 then minor versions are considerd breaking
---@alias TRC_Version {major:integer,minor:integer,patch:integer}

---@generic T
---@alias Maybe
---| "None"
---| {Some: T }

---@alias MoveDir
--- |"Forward"
--- |"Back"
--- |"Up"
--- |"Down"
--- |"Left"
--- |"Right"

---@class pos3: {x:integer,y:integer,z:integer}

---@alias orienation
---Towards -Z
---| "North"
---Towards +X
---| "East"
---Towards +Z
---| "South"
---Towards -X
---| "West"
