#= 
diffs.jl

送信時・受信時におけるパケットの時間間隔を計算しグラフにする

送信時間隔はRTPヘッダーのtimestampから
受信時間隔はrtp_jyushin_maruの受取時タイムスタンプから
それぞれ前のパケットのそれとの差分を計算

=#

using Octo
using Octo.Repo
using Octo.Adapters.PostgreSQL
using Dates
using Plots

struct TestCase
    id::String
    name::String
end

struct RTP
    id::String
    test_case_id::String

    version::Integer
    padding::Bool
    extension::Bool
    csrc_count::Integer
    marker::Bool
    payload_type::Integer
    timestamp::Integer
    ssrc::Integer
    payload::Array{UInt8,1}

    received_at::DateTime

    inserted_at::DateTime
    updated_at::DateTime
end

Schema.model(TestCase, table_name = "test_cases", primary_key = "id")
Schema.model(RTP, table_name = "rtps", primary_key = "id")

Repo.connect(adapter = Octo.Adapters.PostgreSQL,
   dbname = "packet_jyushin_maru_repo",
   user = "postgres",
   password = "postgres",
   host = "localhost",
)


# Main

print("Enter TestCase name: ")
name = readline(stdin)
# name = "2019-01-19 16:00:07.213543Z"

test_case = Repo.get(TestCase, (Name = name,))[1]

tc = from(TestCase)
r = from(RTP)

# rtps = Repo.query([SELECT (r.id,  r.received_at, r.timestamp) FROM r WHERE r.test_case_id == test_case.id ORDER BY r.inserted_at ASC LIMIT 10])
rtps = Repo.query([SELECT (r.serial,  r.received_at, r.timestamp) FROM r WHERE r.test_case_id == test_case.id ORDER BY r.serial ASC OFFSET 40000])

println("Got data, trying to print result...")

# display(rtps)

function convert_to_deltas(rtps)
    R = NamedTuple{(:received_at, :timestamp),Tuple{Integer,Integer}}
    res = R[R((0, 0))]

    for j in 2:size(rtps, 1)
        push!(res, R((rtps[j].received_at - rtps[j - 1].received_at, rtps[j].timestamp - rtps[j - 1].timestamp)))
    end
    return res
end

deltas = convert_to_deltas(rtps)

result = []
for row in deltas
    for key in keys(row)
        push!(result, row[key])
    end
end

println("Converted. Plotting...")

# gr()
# p = plot(reshape(result, :, 2), linewidth = 2, title = name)
# savefig(p, name * ".png")
plotly()
p = plot(reshape(result, :, 2),
    seriestype = :scatter,
    label = ["受信時間隔" "送信時間隔"],
    xlabel = "packet id",
    ylabel = "duration from prev. packet",
    ylims = (-100, 3000),
    yticks = -100:100:3000,
    linewidth = 0, 
    markersize = 1,
    markerstrokewidth = 0,
    title = name,
    size = (1600, 600),
    seriesalpha = 0.5)
display(p)
