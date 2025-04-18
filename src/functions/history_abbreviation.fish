function history_abbreviation
    set path_to_bin /tmp/parser
    set --local indexes ($path_to_bin decode $argv)
    set --local item (get_history_item $indexes[1])
    set --local args (/$path_to_bin parse $indexes[2] $indexes[3] $item)
    if test -z "$args"
        set --show args
        return 1
    else
        set --show args
        echo $args
    end
end
function --local get_history_item
    echo $history[$argv]
end
