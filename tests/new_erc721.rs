use util::must_compile;

mod util;

const bytecode: &'static str = "60003560E01c8063a9059cbb146100a057806342842e0e146101a3578063b88d4fde146101a9578063095ea7b31461027b578063a22cb46514610310578063081812fc146102f357806340c10f19146101af57806370a082311461025e5780636352211e1461039457806306fdde031461035e57806395d89b4114610364578063c87b56dd1461036a57806301ffc9a714610370578063e985e9c514610376575b6044356024356004358083600160005260006020015260406000205491146100c75761019d565b8033146101005733816000526000602001526040600020546101005782600360005260006020015260406000205433146101005761019d565b6001816002600052600060200152604060002054038160026000526000602001526040600020558160026000526000602001526040600020546001018260026000526000602001526040600020558183600160005260006020015260406000205560008360036000526000602001526040600020557fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000a4005b60006000fd5b60006000fd5b60006000fd5b60005433146101be5760006000fd5b6024356004356000826001600052600060200152604060002054156101e257610258565b8160026000526000602001526040600020546001018260026000526000602001526040600020558183600160005260006020015260406000205560008360036000526000602001526040600020557fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60006000a4005b60006000fd5b600435600260005260006020015260406000205460005260206000f35b6024358060016000526000602001526040600020548033143382600052600060200152604060002054176102ae576102ed565b60043580836003600052600060200152604060002055907f8C5BE1E5EBEC7D5BD14F71427D1E84F3DD0314C0F7B2291E5B200AC8C7C3B92560006000a4005b60006000fd5b600435600360005260006020015260406000205460005260206000f35b60243560043533600052600060200152604060002055600435336024356000527f17307EAB39AB6107E8899845AD3D59BD9653F200F220920489CA2B5937696C3160006000a4005b60006000fd5b60006000fd5b60006000fd5b60006000fd5b60006000fd5b60243560043560005260006020015260406000205460005260206000f35b600435600160005260006020015260406000205460005260206000f3";

// huff-examples erc721

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
APPROVAL_FOR_ALL_EVENT_SIGNATURE = 0x17307EAB39AB6107E8899845AD3D59BD9653F200F220920489CA2B5937696C31

NAME_LOCATION = 5
SYMBOL_LOCATION = 4
OWNER_LOCATION = 1
BALANCE_LOCATION = 2 
SINGLE_APPROVAL_LOCATION = 3


CONSTRUCTOR = () {
    OWNABLE_CONSTRUCTOR()
}



BALANCE_OF = () {
    0x04 calldataload                               
    BALANCE_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00) 
    0x00 mstore                                     
    0x20 0x00 return                                
}

OWNER_OF = () {
    0x04 calldataload                               
    OWNER_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00)   
    0x00 mstore                                     
    0x20 0x00 return                                
}

IS_APPROVED_FOR_ALL = () {
    0x24 calldataload               
    0x04 calldataload               
    LOAD_ELEMENT_FROM_KEYS(0x00)    
    
    0x00 mstore
    0x20 0x00 return
}

GET_APPROVED = () {
    0x04 calldataload               
    SINGLE_APPROVAL_LOCATION
    LOAD_ELEMENT_FROM_KEYS(0x00)    
    0x00 mstore
    0x20 0x00 return
}

NAME = () {
    0x00 0x00 revert
}

SYMBOL = () {
    0x00 0x00 revert
}

TOKEN_URI = () {
    0x00 0x00 revert
}

SUPPORTS_INTERFACE = () {
    0x00 0x00 revert
}



TRANSFER_TAKE_FROM = (error) {
    dup1                                            
    dup4                                            
    OWNER_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00)   
    swap2                                           
    eq                                              
    cont jumpi                                      
        error jump
    cont:

    
    dup1 caller                                     
    eq                                              
    is_authorized jumpi                             

    
    caller dup2                                     
    LOAD_ELEMENT_FROM_KEYS(0x00)                    
    is_authorized jumpi                             

    
    dup3                                            
    SINGLE_APPROVAL_LOCATION                      
    LOAD_ELEMENT_FROM_KEYS(0x00)                    
    caller eq                                       
    is_authorized jumpi                             
        error jump                                   
    is_authorized:

    
    0x01 dup2                                       
    BALANCE_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00) 
    sub dup2                                        
    BALANCE_LOCATION
    STORE_ELEMENT_FROM_KEYS(0x00)                   

}

TRANSFER_GIVE_TO = () {
    
    
    dup2                            
	BALANCE_LOCATION
    LOAD_ELEMENT_FROM_KEYS(0x00)    
    0x01 add                        

    
	dup3                            
    BALANCE_LOCATION
    STORE_ELEMENT_FROM_KEYS(0x00)   

    
    dup2                            
    dup4                            
    OWNER_LOCATION
    STORE_ELEMENT_FROM_KEYS(0x00)   

    
    0x00 dup4                       
    SINGLE_APPROVAL_LOCATION
    STORE_ELEMENT_FROM_KEYS(0x00)   
}

APPROVE = () {
    
    0x24 calldataload dup1          
    OWNER_LOCATION
    LOAD_ELEMENT_FROM_KEYS(0x00)    
    dup1 caller                     
    eq                              

    
    caller dup3                     
    LOAD_ELEMENT_FROM_KEYS(0x00)    
    or cont jumpi                   
        error jump
    cont:

    
    0x04 calldataload dup1 dup4     
    SINGLE_APPROVAL_LOCATION
    STORE_ELEMENT_FROM_KEYS(0x00)   
    swap1                           

    
    APPROVAL_EVENT_SIGNATURE                      
    0x00 0x00                                       
    log4                                            

    stop
    error:
        0x00 0x00 revert
}

SET_APPROVAL_FOR_ALL = () {
    0x24 calldataload                               
    0x04 calldataload                               
    caller                                          

    STORE_ELEMENT_FROM_KEYS(0x00)                   

    0x04 calldataload                               
    caller                                          
    0x24 calldataload                               
    
    0x00 mstore                                     
    APPROVAL_FOR_ALL_EVENT_SIGNATURE              
    0x00 0x00                                       
    log4                                            

    stop
    error:
        0x00 0x00 revert
}

TRANSFER_FROM = () {
    
    0x44 calldataload   
    0x24 calldataload   
    0x04 calldataload   

    TRANSFER_TAKE_FROM(error)                       
    TRANSFER_GIVE_TO()                              

    
    TRANSFER_EVENT_SIGNATURE                      
    0x20 0x00                                       
    log4                                            

    stop
    
    error:
        0x00 0x00 revert
}

SAFE_TRANSFER_FROM = () {
    0x00 0x00 revert
}

SAFE_TRANSFER_FROM_WITH_DATA = () {
    0x00 0x00 revert
}

MINT = () {
    
    ONLY_OWNER()

    
    0x24 calldataload                               
    0x04 calldataload                               
    0x00                                            
    dup3                                            

    
    OWNER_LOCATION LOAD_ELEMENT_FROM_KEYS(0x00)   
    iszero cont jumpi
        error jump

    cont:

    
    TRANSFER_GIVE_TO()                              

    
    TRANSFER_EVENT_SIGNATURE                      
    0x00 0x00                                       
    log4                                            

    stop

    error:
        0x00 0x00 revert
}


MAIN = () {
    
    0x00 calldataload 0xE0 shr
    dup1 0xa9059cbb eq transferFrom jumpi
    dup1 0x42842e0e eq safeTransferFrom jumpi
    dup1 0xb88d4fde eq safeTransferFromWithData jumpi
    dup1 0x095ea7b3 eq approve jumpi
    dup1 0xa22cb465 eq setApprovalForAll jumpi
    dup1 0x081812fc eq getApproved jumpi
    dup1 0x40c10f19 eq mint jumpi
    dup1 0x70a08231 eq balanceOf jumpi
    dup1 0x6352211e eq ownerOf jumpi
    dup1 0x06fdde03 eq name jumpi
    dup1 0x95d89b41 eq symbol jumpi
    dup1 0xc87b56dd eq tokenURI jumpi
    dup1 0x01ffc9a7 eq supportsInterface jumpi
    dup1 0xe985e9c5 eq isApprovedForAll jumpi



    transferFrom:
        TRANSFER_FROM()
    safeTransferFrom:
        SAFE_TRANSFER_FROM()    
    safeTransferFromWithData:
        SAFE_TRANSFER_FROM_WITH_DATA()  
    mint:
        MINT()
    balanceOf:
        BALANCE_OF()
    approve:
        APPROVE()
    getApproved:
        GET_APPROVED()
    setApprovalForAll:
        SET_APPROVAL_FOR_ALL()
    name:
        NAME()                  
    symbol:
        SYMBOL()                
    tokenURI:
        TOKEN_URI()             
    supportsInterface:
        SUPPORTS_INTERFACE()    
    isApprovedForAll:
        IS_APPROVED_FOR_ALL()
    ownerOf:
        OWNER_OF()
}
";

#[test]
fn compile_new_erc20() {
    assert_eq!(must_compile(input), bytecode.to_lowercase());
}
