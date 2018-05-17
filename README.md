# pop-pop-ret

## Example 
```
./target/release/pop_pop_ret ~/QubesIncoming/kali64/dll/* -g '\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0b\x0c\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2a\x2b\x2c\x2d\x2e\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3b\x3c\x3d\x3e\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5a\x5b\x5c\x5d\x5e\x5f\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a\x7b\x7c\x7d\x7e\x7f'
Number of virtual core: 8
/home/user/QubesIncoming/kali64/dll/hpi.dll	111b	6d1d111b
/home/user/QubesIncoming/kali64/dll/hpi.dll	116e	6d1d116e
....
```

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

```
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
```
