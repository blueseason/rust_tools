/*
1. 使用二维数组表示状态机,
colomn 为 ascii的字符，共128个，每当有一个新的状态时，插入一个新的 状态机 column
2. 匹配pattern生成状态机，每个匹配的字符的next为下一个column的index，其他为0表示无法跳转到下一个状态
3. 为了支持 * 模式，需要
   3.1 引入offset,表示主串是否需要调整位置
   不匹配，offset为0
   匹配时， next需要循环指向自己，这里的next为状态机的当前位置的跳转位置，所以需要设置为n-1

*/

type FsmIndex = usize;

// the total ascii size
const FSM_COLUMN_SIZE: usize = 130;
const FSM_LINE_END: usize = 129;

#[derive(Clone, Default, Copy)]
struct FsmAction {
    next: FsmIndex,
    offset: i32, //表示主串是否需要调整位置
}
#[derive(Clone)]
struct FsmColumn {
    ts: [FsmAction; FSM_COLUMN_SIZE],
}

impl FsmColumn {
    fn new() -> Self {
        Self {
            ts: [Default::default(); FSM_COLUMN_SIZE],
        }
    }
}

struct Regex {
    cs: Vec<FsmColumn>,
}

impl Regex {
    fn compile(src: &str) -> Self {
        let mut fsm = Self { cs: Vec::new() };
        fsm.cs.push(FsmColumn::new());
        for c in src.chars() {
            let mut col = FsmColumn::new();
            match c {
                '$' => {
                    col.ts[FSM_LINE_END] = FsmAction {
                        next: fsm.cs.len() + 1,
                        offset: 1,
                    };
                    fsm.cs.push(col);
                }
                '.' => {
                    // just match all the printable char
                    for i in 32..127 {
                        col.ts[i] = FsmAction {
                            next: fsm.cs.len() + 1,
                            offset: 1,
                        };
                    }
                    fsm.cs.push(col);
                }
                '*' => {
                    // do not ceate new state and loop to self
                    let n = fsm.cs.len();
                    for t in fsm.cs.last_mut().unwrap().ts.iter_mut() {
                        if t.next == n {
                            t.next = n - 1; // if match,loop to self, next = self.pos
                        } else if t.next == 0 {
                            t.next = n; // not match, jump to next
                            t.offset = 0;
                        } else {
                            unreachable!();
                        }
                    }
                }
                '+' => {
                    let n = fsm.cs.len();

                    fsm.cs.push(fsm.cs.last().unwrap().clone());

                    for t in fsm.cs.last_mut().unwrap().ts.iter_mut() {
                        if t.next == n {
                            // Just leave it as it is. It's already looped.
                        } else if t.next == 0 {
                            t.next = n + 1;
                            t.offset = 0;
                        } else {
                            unreachable!();
                        }
                    }
                }
                _ => {
                    col.ts[c as FsmIndex] = FsmAction {
                        next: fsm.cs.len() + 1,
                        offset: 1,
                    };
                    fsm.cs.push(col);
                }
            }
        }
        fsm
    }

    fn match_str(&self, input: &str) -> bool {
        let mut state = 1;
        let mut head = 0;
        let chars = input.chars().collect::<Vec<_>>();
        let n = chars.len();

        while 0 < state && state < self.cs.len() && head < n {
            let action = self.cs[state].ts[chars[head] as usize];
            state = action.next;
            head = (head as i32 + action.offset) as usize;
        }

        if state == 0 {
            return false;
        }

        if state < self.cs.len() {
            let action = self.cs[state].ts[FSM_LINE_END];
            state = action.next;
        }

        return state >= self.cs.len();
    }

    fn dump(&self) {
        for symbol in 0..FSM_COLUMN_SIZE {
            //03 按3字符对齐
            print!("{:03} => ", symbol);
            for column in self.cs.iter() {
                print!("({},{}) ", column.ts[symbol].next, column.ts[symbol].offset)
            }
            println!();
        }
    }
}

fn main() {
    let src = "a*bc";
    let regex = Regex::compile(src);
    regex.dump();
    println!("------------------------------");

    let inputs = vec![
        "Hello, world",
        "bc",
        "abc",
        "aabc",
        "aaabc",
        "bbc",
        "cbc",
        "cbd",
        "cbt",
        "abcd",
    ];

    println!("Regex: {}", src);
    for input in inputs.iter() {
        println!("{:?}  =>  {:?}", input, regex.match_str(input));
    }
}
