function history_abbreviation
    set --local indexes ($__fish_config_dir/functions/fish_history_abbreviation $argv)
    if test -z "$indexes"
        return 1
    end
    set --local item $history[$indexes[1]]
    set --local tokens (commandline --tokens-raw --input=$item)
    set --local args $tokens[$indexes[2]..$indexes[3]]
    if test -z "$args"
        return 1
    else
        echo $args
    end
end

