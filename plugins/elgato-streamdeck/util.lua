local info = require("info")

local util = {}

---Flip key index for Stream Deck Original
---@param columnCount number
---@param index number
---@return number
function util.flipKeyIndex(columnCount, index)
    local zeroIndex = index - 1
    local col = zeroIndex % columnCount
    return ((zeroIndex - col) + ((columnCount - 1) - col)) + 1
end

---Read button states from data
---@param kind KindInfo
---@param data ByteArray
---@return boolean[]?
function util.readButtonStates(kind, data)
    if data[1] == 0 then
        return nil
    end

    local bools = {}

    if kind.kind == info.ORIGINAL then
        local columns = kind:getColumnCount()

        for i = 1, kind:getKeyCount() do
            local flipped = util.flipKeyIndex(columns, i)

            bools[i] = data[flipped + 1] ~= 0
        end
    elseif kind.kind == info.MINI or kind.kind == info.MINI_MK2 then
        for i = 2, #data do
            bools[i - 1] = data[i] ~= 0
        end
    else
        for i = 5, #data do
            bools[i - 4] = data[i] ~= 0
        end
    end

    return bools
end

---Reads LCD input and returns event data
---@param data ByteArray
---@return string?
---@return table?
function util.readLCDInput(data)
    local startX = byte.u16(data[7], data[8])
    local startY = byte.u16(data[9], data[10])

    if data[5] == 1 then
        return common.touchScreenPressEvent.name, { startX, startY }
    elseif data[5] == 2 then
        return common.touchScreenLongPressEvent.name, { startX, startY }
    elseif data[5] == 3 then
        local endX = byte.u16(data[11], data[12])
        local endY = byte.u16(data[13], data[14])

        return common.touchScreenSwipeEvent.name, { {startX, startY}, {endX, endY} }
    end
end

---Reads encoder input and returns event data
---@param kind KindInfo
---@param data ByteArray
---@return string?
---@return (boolean | number)[]?
function util.readEncoderInput(kind, data)
    if data[5] == 0 then
        local buttonData = {}

        for i = 6, 5 + kind:getEncoderCount() do
            buttonData[i - 5] = data[i] ~= 0
        end

        return common.buttonEvent.name, buttonData
    elseif data[5] == 1 then
        local twistData = {}

        for i = 6, 5 + kind:getEncoderCount() do
            twistData[i - 5] = byte.i8(data[i])
        end

        return common.encoderEvent.name, twistData
    end
end

return util