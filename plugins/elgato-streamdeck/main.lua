---@diagnostic disable: cast-local-type
local info = require("info")
local util = require("util")

---@type LuaDriver
local driver = {}

function driver:listDevices()
    if hid:timeSinceRefresh() > 5 then
        hid:refresh()
    end

    local result = {}

    for _, device in pairs(hid:list(info.elgatoVID)) do
        if info.isStreamDeck(device.product_id) then
            local entry = {
                device.serial_number,
                device.product_string,
                device.product_id
            }

            table.insert(result, entry)
        end
    end

    return result
end

function driver:connectDevice(id)
    local pid = id[3]
    local kind = info.fromPID(pid)

    local ok, streamdeck = pcall(hid.open, hid, info.elgatoVID, pid, id[1])

    if not ok or not kind then
        return nil
    end

    ---@type LuaDevice
    local device = {}

    function device:disconnect()
        streamdeck:destroy()
    end

    function device:layout()
        return kind:getLayout()
    end

    local lastButtonState = {}
    local function compareButtonStates(events, newButtonState)
        if not newButtonState then
            return
        end

        for i = 1, #newButtonState do
            local old, new = lastButtonState[i], newButtonState[i]

            if (not not old) ~= new then
                events:input(i, common.buttonEvent.name, new == true)
            end
        end

        lastButtonState = newButtonState
    end

    local lastEncoderButtonState = {}

    function device:tick(events)
        if kind.kind == info.PLUS then
            local _, data = streamdeck:read(math.max(14, 5 + kind:getEncoderCount()))

            if data[1] == 0 then
                return
            end

            if data[2] == 0 then
                local newButtonState = util.readButtonStates(kind, data)
                compareButtonStates(events, newButtonState)
            elseif data[2] == 2 then
                local name, eventData = util.readLCDInput(data)

                if not name or not data then
                    return
                end

                events:input(9, name, eventData)
            elseif data[2] == 3 then
                local name, state = util.readEncoderInput(kind, data)

                if not name or not state then
                    return
                end

                local keyCount = kind:getKeyCount()

                if name == common.buttonEvent.name then
                    for i = 1, #state do
                        local old, new = lastEncoderButtonState[i], state[i]

                        if (not not old) ~= new then
                            events:input(keyCount + i + 1, common.buttonEvent.name, new == true)
                        end
                    end

                    lastEncoderButtonState = state
                else
                    for i = 1, #state do
                        local num = state[i]

                        if num ~= 0 then
                            events:input(keyCount + i + 1, common.encoderEvent.name, num)
                        end
                    end
                end
            end
        else
            local paddingChoice = kind.kind == info.ORIGINAL or kind.kind == info.MINI or kind.kind == info.MINI_MK2

            local _, data = streamdeck:read((paddingChoice and 1 or 4) + kind:getKeyCount() + kind:getTouchpointCount())

            if data[1] == 0 then
                return
            end

            local newButtonState = util.readButtonStates(kind, data)
            compareButtonStates(events, newButtonState)
        end
    end

    return device
end

---@param reg PluginRegistry
function Register(reg)
    reg:registerDriver("driver", driver)
end