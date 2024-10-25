---@meta

---@class ComputerSetupToServerPacket
---@field world string
---@field id integer

---@alias TurtleToServerPacket
---| [integer,]
---| [Pos3,]
---| [Orientation,]
---| [MoveDirection,]
---| [integer,]
---| { UpdateBlocks: {up: string | nil, down: string | nil, front: string | nil, } }
---| { UpdateSlotContents: {slot: integer, contents: Item | nil, } }
---| [ComputerToServerPacket,]

---@alias ComputerToServerPacket
---| [string,]
---| [string[],]
---| [string,]
---| [string,]
---| "Ping"