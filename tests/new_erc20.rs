use util::must_compile;

mod util;

const bytecode: &'static str = "60003560E01c8063a9059cbb1461004857806340c10f19146100de57806370a082311461014e57806318160ddd1461016b578063095ea7b314610177578063dd62ed3e1461018e575b600435336024358160016000526000602001526040600020548082116100d8578190038260016000526000602001526040600020558281906001600052600060200152604060002054018360016000526000602001526040600020556000527fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000a3600160005260206000f35b60006000fd5b60005433146100ed5760006000fd5b600435600060243582819060016000526000602001526040600020540183600160005260006020015260406000205580600254016002556000527fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000a35b600435600160005260006020015260406000205460005260206000f35b60025460005260206000f35b602435600435336000526000602001526040600020555b60243560043560005260006020015260406000205460005260206000f3";

// huff-examples erc20

const input: &'static str = "
OWNER_POINTER = 0

OWNABLE_CONSTRUCTOR = () {
    caller OWNER_POINTER sstore
}

OWNABLE_SET_OWNER = () {
    OWNER_POINTER sstore
}

OWNABLE_GET_OWNER = () {
    OWNER_POINTER sload
}

ONLY_OWNER = () {
	OWNER_POINTER sload caller eq is_owner jumpi
		0x00 0x00 revert
	is_owner:
}

IS_CONTRACT = () {
    extcodesize
}

SEND_ETH = () {
    0x00    
    0x00    
    0x00    
    0x00    
    dup5    
    dup7    
}

MASK_ADDRESS = () {
    0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff
	and
}



UTILS__NOT_PAYABLE = (error_location) {
	callvalue error_location jumpi
}




GET_SLOT_FROM_KEY = (mem_ptr) {
    
    
    mem_ptr
    mstore      
    
    
    0x20                
    mem_ptr
    sha3                
}



GET_SLOT_FROM_KEYS = (mem_ptr) {
    mem_ptr
    mstore              
    mem_ptr 0x20 add
    mstore              
    
    0x40
    mem_ptr
    sha3        
}

LOAD_ELEMENT = (mem_ptr) {
    
    GET_SLOT_FROM_KEY(mem_ptr)
    sload                       
}

LOAD_ELEMENT_FROM_KEYS = (mem_ptr) {
    GET_SLOT_FROM_KEYS(mem_ptr)
    sload                       
}

STORE_ELEMENT = (mem_ptr) {
    GET_SLOT_FROM_KEY(mem_ptr)
    sstore                      
}

STORE_ELEMENT_FROM_KEYS = (mem_ptr) {
    
    GET_SLOT_FROM_KEYS(mem_ptr)
    sstore                      
}

TRANSFER_EVENT_SIGNATURE = 0xDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF
APPROVAL_EVENT_SIGNATURE = 0x8C5BE1E5EBEC7D5BD14F71427D1E84F3DD0314C0F7B2291E5B200AC8C7C3B925

BALANCE_LOCATION = 1
ALLOWANCE_LOCATION = 3
TOTAL_SUPPLY_LOCATION = 2

CONSTRUCTOR = () {
    OWNABLE_CONSTRUCTOR()
}

BALANCE_OF = () {
    0x04 calldataload                               
    BALANCE_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00)
    0x00 mstore                                     
    0x20 0x00 return                                
}

TOTAL_SUPPLY = () {
    TOTAL_SUPPLY_LOCATION sload
    0x00 mstore                     
    0x20 0x00 return                
}

ALLOWANCE = () {
    0x24 calldataload               
    0x04 calldataload               
    LOAD_ELEMENT_FROM_KEYS(0x00)    

    0x00 mstore
    0x20 0x00 return
}


TRANSFER_TAKE_FROM = (error) {
    
    
    dup2                
    BALANCE_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00)
    dup1                
    dup3                
    gt                  
    error jumpi

    
    
    dup2                
    swap1               
    sub                 
    dup3                
    BALANCE_LOCATION STORE_ELEMENT_FROM_KEYS(0x00)
}

TRANSFER_GIVE_TO = () {
    dup3
    dup2                
    swap1               
    BALANCE_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00)
    add                 
    dup4                
    BALANCE_LOCATION STORE_ELEMENT_FROM_KEYS(0x00)
}

APPROVE = () {
    0x24 calldataload       
    0x04 calldataload       
    caller                  

    STORE_ELEMENT_FROM_KEYS(0x00)
}

TRANSFER = () {
    
    0x04 calldataload   
    caller              
    0x24 calldataload   

    
    TRANSFER_TAKE_FROM(error)   
    TRANSFER_GIVE_TO()          

    
    0x00 mstore                 
    TRANSFER_EVENT_SIGNATURE
    0x20 0x00                   
    log3                        

    
    0x01 0x00 mstore
    0x20 0x00 return

    
    error:
        0x00 0x00 revert
}

MINT = () {
    
    ONLY_OWNER()

    
    0x04 calldataload   
    0x00                
    0x24 calldataload   

    
    TRANSFER_GIVE_TO()  

    
    dup1                            
    TOTAL_SUPPLY_LOCATION sload
    add                             
    TOTAL_SUPPLY_LOCATION sstore


    
    0x00 mstore                 
    TRANSFER_EVENT_SIGNATURE
    0x20 0x00                   
    log3                        
}


MAIN = () {
    
    0x00 calldataload 0xE0 shr
    dup1 0xa9059cbb eq transfer jumpi
    dup1 0x40c10f19 eq mints jumpi
    dup1 0x70a08231 eq balanceOf jumpi
    dup1 0x18160ddd eq totalSupply jumpi
    dup1 0x095ea7b3 eq approve jumpi
    dup1 0xdd62ed3e eq allowance jumpi

    transfer:
        TRANSFER()
    mints:
        MINT()
    balanceOf:
        BALANCE_OF()
    totalSupply:
        TOTAL_SUPPLY()
    approve:
        APPROVE()
    allowance:
        ALLOWANCE()

}
";

#[test]
fn compile_new_erc20() {
    assert_eq!(must_compile(input), bytecode.to_lowercase());
}
