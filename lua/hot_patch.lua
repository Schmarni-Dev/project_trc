local f = fs.open("startup.lua", "w")
if f == nil then
    print("Well Fuck")
    return
end
f.write("os.run({},\"run.lua\")")
f.close()
