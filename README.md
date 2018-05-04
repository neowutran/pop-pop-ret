# pop-pop-ret

## Example 

[user@dev pop_pop_ret]$ ./target/release/pop_pop_ret ~/QubesIncoming/kali64/ov.dll 
Number of virtual core: 8
/home/user/QubesIncoming/kali64/ov.dll:24da
/home/user/QubesIncoming/kali64/ov.dll:32ec
/home/user/QubesIncoming/kali64/ov.dll:330e
/home/user/QubesIncoming/kali64/ov.dll:332d
/home/user/QubesIncoming/kali64/ov.dll:334f
/home/user/QubesIncoming/kali64/ov.dll:ecdd
/home/user/QubesIncoming/kali64/ov.dll:102f5
/home/user/QubesIncoming/kali64/ov.dll:10a33
/home/user/QubesIncoming/kali64/ov.dll:10f34
/home/user/QubesIncoming/kali64/ov.dll:1124c
/home/user/QubesIncoming/kali64/ov.dll:11278
/home/user/QubesIncoming/kali64/ov.dll:11282
/home/user/QubesIncoming/kali64/ov.dll:11793
/home/user/QubesIncoming/kali64/ov.dll:11a40
/home/user/QubesIncoming/kali64/ov.dll:14e65
/home/user/QubesIncoming/kali64/ov.dll:15560
/home/user/QubesIncoming/kali64/ov.dll:16513
/home/user/QubesIncoming/kali64/ov.dll:16520
/home/user/QubesIncoming/kali64/ov.dll:16d33
/home/user/QubesIncoming/kali64/ov.dll:173d6
/home/user/QubesIncoming/kali64/ov.dll:1796f
/home/user/QubesIncoming/kali64/ov.dll:179b6
/home/user/QubesIncoming/kali64/ov.dll:185ed
/home/user/QubesIncoming/kali64/ov.dll:1b0a8
/home/user/QubesIncoming/kali64/ov.dll:1b1a4
/home/user/QubesIncoming/kali64/ov.dll:1b1ae
/home/user/QubesIncoming/kali64/ov.dll:1ee53
/home/user/QubesIncoming/kali64/ov.dll:1ef67
/home/user/QubesIncoming/kali64/ov.dll:1f277
/home/user/QubesIncoming/kali64/ov.dll:243d6
/home/user/QubesIncoming/kali64/ov.dll:2643a
/home/user/QubesIncoming/kali64/ov.dll:2899f
/home/user/QubesIncoming/kali64/ov.dll:2a0ff
/home/user/QubesIncoming/kali64/ov.dll:2a3cf
/home/user/QubesIncoming/kali64/ov.dll:2a3ea
/home/user/QubesIncoming/kali64/ov.dll:2a446
/home/user/QubesIncoming/kali64/ov.dll:2ea04
/home/user/QubesIncoming/kali64/ov.dll:2eda3
/home/user/QubesIncoming/kali64/ov.dll:2ef74


# Bonus: Bash version

````bash
#!/bin/bash
# http://www.mathemainzel.info/files/x86asmref.html
# http://www.sparksandflames.com/files/x86InstructionChart.html
opcode_pop='(07|17|1F|58|59|5A|5B|5C|5D|5E|5F)'
opcode_ret='(C2|C3|CB|CA)'
hex=$(xxd -p $1 | tr -d '\n')

opcode_pop=${opcode_pop^^}
opcode_ret=${opcode_ret^^}
hex=${hex^^}

result=$(echo "$hex" | grep -aEbo "$opcode_pop{2}$opcode_ret" | sed -E 's/:[^:]+$//g')
while read -r line; do
    offset1=${line#0}
    offset2=$(( offset1 / 2 ))
    printf '%x\n' $offset2
done <<< "$result"
````

## Example

./findjump.sh ov.dll 
24da
32ec
330e
332d
334f
ecdd
102f5
10a33
10f34
1124c
11278
11282
11793
11a40
14e65
15560
16513
16520
16d33
173d6
1796f
179b6
185ed
1b0a8
1b1a4
1b1ae
1ee53
1ef67
1f277
243d6
2643a
2899f
2a0ff
2a3cf
2a3ea
2a446
2ea04
2eda3
2ef74
