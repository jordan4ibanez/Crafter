--[[

    This is the literal entry point into the lua scope.

    This is why it's called "lua_context.lua". Without this file, the script api does not have anything to work from.

    From here you can freely modify your game to be whatever you want it to be.

    But for now it is going to be Crafter.

]]--

require("lua_libraries.lua_helpers")

--[[
    This is the base building block of the entire Crafter api.

    Everything from here on out is contained within this table.

    Localizing functions from this table can greatly improve your performance.
]]--

crafter = {
    blocks = {},
    -- localization cached & cached into table
    get_operating_system = get_operating_system
}


print("lua operating system detection: " .. crafter.get_operating_system());