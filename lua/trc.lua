---Follows SemVer, if major = 0 then minor versions are considerd breaking
---@diagnostic disable-next-line: duplicate-doc-alias
---@alias TRC_Version {major:integer,minor:integer,patch:integer}

---@alias TrcExtensions "trc_position_tracking" | "trc_pathfinding"

for i, arg in ipairs(...) do
    print("test ", arg)
end

-- local function new_extensions()
--     ---@class TrcExtensionList
--     ---@field private extensions table<string>
--    local extensions = {}
--     function extensions:push()
--
--     end
-- end


-- if TRC_INTERNAL_STD_IMPL ~= nil then
--     if TRC_INTERNAL_STD_IMPL.version.major ~= loader_version.major then
--         error("INCOMPATIBLE_TRC_VERSION: major version mismatch! service major is \'" ..
--             TRC_INTERNAL_STD_IMPL.version.major .. "\' and loader major is \'" .. loader_version.major .. "\'")
--     end
--     if TRC_INTERNAL_STD_IMPL.version.major == 0 and TRC_INTERNAL_STD_IMPL.version.minor ~= loader_version.minor then
--         error("INCOMPATIBLE_TRC_VERSION: major version is 0, minor version mismatch! service minor is \'" ..
--             TRC_INTERNAL_STD_IMPL.version.minor .. "\' and loader minor is \'" .. loader_version.minor .. "\'")
--     end
--     if TRC_INTERNAL_STD_IMPL.version.major == 0 and TRC_INTERNAL_STD_IMPL.version.minor == 0 and TRC_INTERNAL_STD_IMPL.version.patch ~= loader_version.patch then
--         error("INCOMPATIBLE_TRC_VERSION: major and minor versions are 0, patch version mismatch! service patch is \'" ..
--             TRC_INTERNAL_STD_IMPL.version.patch .. "\' and loader patch is \'" .. loader_version.patch .. "\'")
--     end
--
--     return TRC_INTERNAL_STD_IMPL.impl
-- end

TRC_INTERNAL_API = {}

---@return TRC_Version
function TRC_INTERNAL_API.get_version()
    return { major = 0, minor = 0, patch = 1 }
end

---@return string[]
function TRC_INTERNAL_API:get_available_extensions()
    return {}
end
