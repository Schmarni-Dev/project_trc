if log == nil then
    ---@diagnostic disable-next-line: lowercase-global
    log = print
end

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

---Creates a maybe table for json
---@generic T
---@param val T | nil
---@return Maybe<T>
local function maybe(val)
    if type(val) == "nil" then
        return "None"
    end
    return { Some = val }
end

---@generic T: (number | integer)
---@param num T|"unlimited"
---@return T
local function fix_num_or_unlimited(num)
    if num == "unlimited" then
        return -1
    end
    return num
end

---@generic T
---@param val T
---@param bool boolean
---@return Maybe<T>
--
local function get_maybe_using_bool(bool, val)
    local data = val
    if bool == false then
        data = nil
    end
    return maybe(data)
end

--#region Packet Build Functions
---@param items Maybe<turtleDetails>[]
---@param name string
---@param index integer
---@param fuel number | "unlimited"
---@param max_fuel integer | "unlimited"
local function ConstructInfoPacket(items, name, index, fuel, max_fuel)
    return {
        Info = {
            inventory = { inv = items },
            name = name,
            index = index,
            fuel = fix_num_or_unlimited(fuel),
            max_fuel = fix_num_or_unlimited(max_fuel)
        }
    }
end

---@param up Maybe<string>
---@param down Maybe<string>
---@param front Maybe<string>
local function ConstructBlocksPacket(up, down, front)
    return { Blocks = { up, down, front } }
end

---@param data string | inspectInfo
---@return string
local function fix_inspect(data)
    if type(data) == "string" then
        return data
    end
    return data.name
end

---@param msg table | string
local function print_msg(msg)
    if type(msg) == "string" then
        log(msg)
    else
        log(textutils.serialize(msg))
    end
end

---@return string
local function get_label()
    local label = os.getComputerLabel()
    if label == nil then
        label = ""
    end
    return label
end

---@param exits boolean
---@param data string | inspectInfo
---@return Maybe<string>
local function process_inspect(exits, data)
    return get_maybe_using_bool(exits, fix_inspect(data))
end

---@class Queue<T>: { [ integer ]:T, first: integer, last: integer, push: fun(self: Queue<T>,item: T), pop_handler: fun(self: Queue<T>,callback: fun(value: T)), get_amount_in_queue: fun(self: Queue<T>): integer }


---@generic T
---@return Queue<T>
local function new_queue()
    ---@generic T
    ---@type Queue<T>
    local Queue = { first = 0, last = 0 }

    ---@generic T
    ---@param item T
    function Queue:push(item)
        local last = self.last + 1
        self.last = last
        self[last] = item
    end

    ---@return integer
    function Queue:get_amount_in_queue()
        return (self.last + 1) - self.first
    end

    ---@generic T
    ---@param callback fun(value: T)
    function Queue:pop_handler(callback)
        local first = self.first
        if first > self.last then
            return
        end
        local value = self[first]
        self.first = first + 1
        if value ~= nil then
            log(value)
            callback(value)
        end
    end

    return Queue
end

---@param f fun()
---@param dont_yield boolean?
---@return fun()
local function loop(f, dont_yield)
    local dy = dont_yield or false
    return function()
        while true do
            f()
            if not dy then
                coroutine.yield()
            end
        end
    end
end

---@param logo string
---@param spacer string
---@return string
local function get_logo_string(logo, spacer)
    local width = term.getSize()
    local adjusted_width = width - logo:len()
    local pad_string = string.rep(spacer, adjusted_width / (spacer:len() * 2))
    return pad_string .. logo .. pad_string
end

---@param f fun()
local function set_terminate_handler(f)
    ---@diagnostic disable-next-line: lowercase-global
    pull = os.pullEvent
    local function handler(e)
        local event, data = os.pullEventRaw(e)
        if event == "terminate" then
            f()
        else
            return event, data
        end
    end
    os.pullEvent = handler
end

local function reset_event_handler()
    os.pullEvent = pull
end

local function term_clear()
    term.clear()
    term.setCursorPos(1, 1)
end

return {
    maybe = maybe,
    ConstructBlocksPacket = ConstructBlocksPacket,
    ConstructInfoPacket = ConstructInfoPacket,
    process_inspect = process_inspect,
    get_label = get_label,
    fix_inspect = fix_inspect,
    fix_num_or_unlimited = fix_num_or_unlimited,
    print_msg = print_msg,
    get_maybe_using_bool = get_maybe_using_bool,
    new_queue = new_queue,
    loop = loop,
    get_logo_string = get_logo_string,
    set_terminate_handler = set_terminate_handler,
    term_clear = term_clear,
    reset_event_handler = reset_event_handler,
}
