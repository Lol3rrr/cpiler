fn main() -> I32
  block 0x600001b8c110
    Jump 0x600001b8c210

  block 0x600001b8c210
    x_2548118702385322313@0(I32) = (I32) I64(10)
    Save-Variable x_2548118702385322313@0(I32)
    Jump 0x600001b8c290

  block 0x600001b8c290
    x_2548118702385322313@5(I32) = Unknown
    x_2548118702385322313@6(I32) = Unknown
    x_2548118702385322313@1(I32) = Variable(Variable { name: "x_2548118702385322313", generation: 6, ty: I32, meta: Empty, global: false, description: None, current_version: 8 })
    x_2548118702385322313@2(I32) = Variable(Variable { name: "x_2548118702385322313", generation: 1, ty: I32, meta: Empty, global: false, description: None, current_version: 8 })
    __t_0@0(I64) = x_2548118702385322313@2(I32) Logic(Greater) I64(0)
    JumpTrue 0x600001b8c310 if __t_0@0(I64)
    Jump 0x600001b8c390

  block 0x600001b8c390
    __t_3@0(I32) = Variable(Variable { name: "x_2548118702385322313", generation: 5, ty: I32, meta: Empty, global: false, description: None, current_version: 8 })
    Return __t_3@0(I32)

  block 0x600001b8c310
    __t_1@0(I64) = (I64) x_2548118702385322313@5(I32)
    __t_2@0(I64) = __t_1@0(I64) Arith(Sub) I64(1)
    x_2548118702385322313@3(I32) = (I32) __t_2@0(I64)
    Save-Variable x_2548118702385322313@3(I32)
    Jump 0x600001b8c290