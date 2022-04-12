fn main() -> I32
  block 0x600001438090
    Jump 0x600001438110

  block 0x600001438110
    result_5210440206002143880@0(I32) = (I32) I64(100)
    Save-Variable result_5210440206002143880@0(I32)
    Jump 0x600001438190

  block 0x600001438190
    i_2331337243268755618@0(I32) = (I32) I64(0)
    Save-Variable i_2331337243268755618@0(I32)
    Jump 0x600001438210

  block 0x600001438210
    i_2331337243268755618@6(I32) = Unknown
    i_2331337243268755618@1(I32) = Variable(Variable { name: "i_2331337243268755618", generation: 6, ty: I32, meta: Empty, global: false, description: None, current_version: 11 })
    i_2331337243268755618@2(I32) = Variable(Variable { name: "i_2331337243268755618", generation: 1, ty: I32, meta: Empty, global: false, description: None, current_version: 11 })
    __t_0@0(I64) = i_2331337243268755618@2(I32) Logic(Less) I64(10)
    result_5210440206002143880@5(I32) = Unknown
    result_5210440206002143880@6(I32) = Unknown
    JumpTrue 0x600001438290 if __t_0@0(I64)
    Jump 0x600001438310

  block 0x600001438310
    Jump 0x600001438610

  block 0x600001438610
    __t_6@0(I32) = Variable(Variable { name: "result_5210440206002143880", generation: 5, ty: I32, meta: Empty, global: false, description: None, current_version: 10 })
    Return __t_6@0(I32)

  block 0x600001438290
    Jump 0x600001438390

  block 0x600001438390
    j_18075517420823836814@0(I32) = (I32) I64(0)
    Save-Variable j_18075517420823836814@0(I32)
    Jump 0x600001438410

  block 0x600001438410
    j_18075517420823836814@5(I32) = Unknown
    j_18075517420823836814@6(I32) = Unknown
    j_18075517420823836814@1(I32) = Variable(Variable { name: "j_18075517420823836814", generation: 6, ty: I32, meta: Empty, global: false, description: None, current_version: 8 })
    j_18075517420823836814@2(I32) = Variable(Variable { name: "j_18075517420823836814", generation: 1, ty: I32, meta: Empty, global: false, description: None, current_version: 8 })
    __t_1@0(I64) = j_18075517420823836814@2(I32) Logic(Less) I64(10)
    result_5210440206002143880@7(I32) = Unknown
    result_5210440206002143880@8(I32) = Unknown
    JumpTrue 0x600001438490 if __t_1@0(I64)
    i_2331337243268755618@1(Void) = Unknown
    i_2331337243268755618@7(I32) = Unknown
    Jump 0x600001438510

  block 0x600001438510
    Jump 0x600001438590

  block 0x600001438590
    i_2331337243268755618@4(I32) = UnaryOp { op: Arith(Increment), base: Variable(Variable { name: "i_2331337243268755618", generation: 1, ty: Void, meta: Temporary, global: false, description: None, current_version: 2 }) }
    Save-Variable i_2331337243268755618@4(I32)
    Jump 0x600001438210

  block 0x600001438490
    Save-Variable i_2331337243268755618@1(Void)
    __t_2@0(I64) = (I64) result_5210440206002143880@7(I32)
    Save-Variable __t_2@0(I64)
    __t_3@0(I64) = __t_2@0(I64) Arith(Sub) I64(1)
    Save-Variable __t_3@0(I64)
    result_5210440206002143880@3(I32) = (I32) __t_3@0(I64)
    Save-Variable result_5210440206002143880@3(I32)
    j_18075517420823836814@3(I32) = UnaryOp { op: Arith(Increment), base: Variable(Variable { name: "j_18075517420823836814", generation: 5, ty: I32, meta: Empty, global: false, description: None, current_version: 8 }) }
    Save-Variable j_18075517420823836814@3(I32)
    Jump 0x600001438410