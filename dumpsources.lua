local bit14 = function(val1, val2)
    if val2 then
        -- convert from 2 7-bit numbers to 1 14-bit number
        return (val1 << 7) | val2
    else
        -- convert from 1 14-bit number to 2 7-bit numbers
        local msb = (val1 & 0x3F80) >> 7
        local lsb = (val1 & 0x007F)
        return msb, lsb
    end
end

local makeChannel = function(_, msb, lsb)
    return ("ID(0x%02X, 0x%02X)"):format(msb, lsb)
end

local sources = {}

local s = ""


-- Input sources
do
    local auxCurrent = bit14(0x00, 0x44)
    print("auxCurrent: ", auxCurrent)
    local fxCurrent = bit14(0x0C, 0x14)

    sources.input = {}

    for ip=1, 48 do
        local muteLSB = ip-1

        sources.input[ip] = {}

        -- LR target (level, pan, assign)
        sources.input[ip].lr = makeChannel(muteLSB, 0x00, ip-1)
        s = s .. ("(Input(%d), LR) => ID(0x%02X, 0x%02X)"):format(ip, 0x00, ip-1) .. ",\n"

        -- Aux target (level, pan, assign)
        sources.input[ip].aux = {}
        for aux=1, 12 do
            sources.input[ip].aux[aux] = makeChannel(muteLSB, bit14(auxCurrent))
            s = s .. ("(Input(%d), Aux(%d)) => ID(0x%02X, 0x%02X)"):format(ip, aux, bit14(auxCurrent)) .. ",\n"
            auxCurrent = auxCurrent + 1
        end

        -- FX send target (level)
        sources.input[ip].fxSend = {}
        for fx=1, 4 do
            sources.input[ip].fxSend[fx] = makeChannel(muteLSB, bit14(fxCurrent))
            s = s .. ("(Input(%d), FXSend(%d)) => ID(0x%02X, 0x%02X)"):format(ip, fx, bit14(fxCurrent)) .. ",\n"
            fxCurrent = fxCurrent + 1
        end

        -- @Todo (maybe): groups (assign only)
    end
end


-- Group sources
do
    local auxCurrent = bit14(0x05, 0x04)
    local fxCurrent = bit14(0x0D, 0x54)
    local mtxCurrent = bit14(0x0E, 0x4B)

    sources.group = {}

    for group=1, 12 do
        local muteLSB = 0x2F + group

        sources.group[group] = {}

        -- LR target (level, pan, assign)
        sources.group[group].lr = makeChannel(muteLSB, 0x00, 0x2F + group)
        s = s .. ("(Group(%d), LR) => ID(0x%02X, 0x%02X)"):format(group, 0x00, 0x2F + group) .. ",\n"

        -- Aux target (level, pan, assign)
        sources.group[group].aux = {}
        for aux=1, 12 do
            -- This condition is to disallow impossible group/aux combinations
            if group + aux <= 12 then
                sources.group[group].aux[aux] = makeChannel(muteLSB, bit14(auxCurrent))
                s = s .. ("(Group(%d), Aux(%d)) => ID(0x%02X, 0x%02X)"):format(group, aux, bit14(auxCurrent)) .. ",\n"
            end

            auxCurrent = auxCurrent + 1
        end

        -- FX send target (level)
        sources.group[group].fxSend = {}
        for fx=1, 4 do
            sources.group[group].fxSend[fx] = makeChannel(muteLSB, bit14(fxCurrent))
            s = s .. ("(Group(%d), FXSend(%d)) => ID(0x%02X, 0x%02X)"):format(group, fx, bit14(fxCurrent)) .. ",\n"
            fxCurrent = fxCurrent + 1
        end

        -- Matrix targets
        sources.group[group].mtx = {}
        for mtx=1, 3 do
            sources.group[group].mtx[mtx] = makeChannel(muteLSB, bit14(mtxCurrent))
            s = s .. ("(Group(%d), Mtx(%d)) => ID(0x%02X, 0x%02X)"):format(group, mtx, bit14(mtxCurrent)) .. ",\n"
            mtxCurrent = mtxCurrent + 1
        end
    end
end


-- FX return sources
do
    local auxCurrent = bit14(0x06, 0x14)
    local groupCurrent = bit14(0x0B, 0x34)
    local fxCurrent = bit14(0x0E, 0x04)

    sources.fxReturn = {}

    for fx=1, 8 do
        local muteLSB = 0x3B + fx

        sources.fxReturn[fx] = {}

        -- LR target (level, pan, assign)
        sources.fxReturn[fx].lr = makeChannel(muteLSB, 0x00, 0x3B + fx)
        s = s .. ("(FXRet(%d), LR) => ID(0x%02X, 0x%02X)"):format(fx, 0x00, 0x3B + fx) .. ",\n"

        -- Aux target (level, pan, assign)
        sources.fxReturn[fx].aux = {}
        for aux=1, 12 do
            sources.fxReturn[fx].aux[aux] = makeChannel(muteLSB, bit14(auxCurrent))
            s = s .. ("(FXRet(%d), Aux(%d)) => ID(0x%02X, 0x%02X)"):format(fx, aux, bit14(auxCurrent)) .. ",\n"
            auxCurrent = auxCurrent + 1
        end

        -- Group targets (level, pan, assign)
        sources.fxReturn[fx].group = {}
        for group=1, 12 do
            sources.fxReturn[fx].group[group] = makeChannel(muteLSB, bit14(groupCurrent))
            s = s .. ("(FXRet(%d), Group(%d)) => ID(0x%02X, 0x%02X)"):format(fx, group, bit14(groupCurrent)) .. ",\n"
            groupCurrent = groupCurrent + 1
        end

        -- FX send target (level)
        sources.fxReturn[fx].fxSend = {}
        for fxSend=1, 4 do
            sources.fxReturn[fx].fxSend[fxSend] = makeChannel(muteLSB, bit14(fxCurrent))
            s = s .. ("(FXRet(%d), FXSend(%d)) => ID(0x%02X, 0x%02X)"):format(fx, fxSend, bit14(fxCurrent)) .. ",\n"
            fxCurrent = fxCurrent + 1
        end
    end
end


-- FX send sources
do
    sources.fxSend = {}
    for fx=1, 4 do
        -- Output target (level, pan)
        sources.fxSend[fx] = makeChannel(0x50 + fx, 0x0F, 0x0C + fx)
        s = s .. ("(FXSend(%d), Output) => ID(0x%02X, 0x%02X)"):format(fx, 0x0F, 0x0C + fx) .. ",\n"
    end
end


-- LR & aux sources
do
    local mtxCurrent = bit14(0x0E, 0x24)

    -- LR source
    local muteLSB = 0x44

    sources.lr = {}

    -- Matrix targets (level, pan, assign)
    sources.lr.mtx = {}
    for mtx=1, 3 do
        sources.lr.mtx[mtx] = makeChannel(muteLSB, bit14(mtxCurrent))
        s = s .. ("(LR, Mtx(%d)) => ID(0x%02X, 0x%02X)"):format(mtx, bit14(mtxCurrent)) .. ",\n"
        mtxCurrent = mtxCurrent + 1
    end

    -- Output target (level, pan)
    sources.lr.output = makeChannel(muteLSB, 0x0F, 0x00)
    s = s .. ("(LR, Output) => ID(0x%02X, 0x%02X)"):format(0x0F, 0x00) .. ",\n"


    -- Aux sources
    sources.aux = {}
    for aux=1, 12 do
        sources.aux[aux] = {}

        -- Matrix targets (level, pan, assign)
        sources.aux[aux].mtx = {}
        for mtx=1, 3 do
            sources.aux[aux].mtx[mtx] = makeChannel(muteLSB, bit14(mtxCurrent))
            s = s .. ("(Aux(%d), Mtx(%d)) => ID(0x%02X, 0x%02X)"):format(aux, mtx, bit14(mtxCurrent)) .. ",\n"
            mtxCurrent = mtxCurrent + 1
        end

        -- Output target (level, pan)
        sources.aux[aux].output = makeChannel(muteLSB, 0x0F, aux)
        s = s .. ("(Aux(%d), Output) => ID(0x%02X, 0x%02X)"):format(aux, 0x0F, aux) .. ",\n"
    end
end


-- Matrix sources
do
    sources.mtx = {}
    for mtx=1, 3 do
        -- Output target (level, pan)
        sources.mtx[mtx] = makeChannel(0x54 + mtx, 0x0F, 0x10 + mtx)
        s = s .. ("(Mtx(%d), Output) => ID(0x%02X, 0x%02X)"):format(mtx, 0x0F, 0x10 + mtx) .. ",\n"
    end
end

return s
