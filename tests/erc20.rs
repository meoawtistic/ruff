use util::must_compile;

mod util;

const bytecode: &'static str = "60003560e01c8063095ea7b31461026257806318160ddd146102e057806323b872dd146100e557806340c10f19146102f157806370a08231146101b5578063a9059cbb1461005d578063dd62ed3e146101f1573461037f5760006000f35b6004357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff16337fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF6020600060243585600052604060002080548201808311915533600052604060002080548381038492551034171761037f57600052a3600160005260206000f35b6024357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff166004357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff167fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF6020600060443585600052604060002080548201808311915585600052604060002080548381038492551086600052600360205260406000206020523360005260406000208054848103859255103417171761037f57600052a3600160005260206000f35b3461037f576004357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff1660005260406000205460005260206000f35b3461037f576004357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff16600052600360205260406000206020526024357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff1660005260406000205460005260206000f35b3461037f576004357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff16337f8C5BE1E5EBEC7D5BD14F71427D1E84F3DD0314C0F7B2291E5B200AC8C7C3B9256020600060243580336000526003602052604060002060205286600052604060002055600052a3600160005260206000f35b3461037f5760025460005260206000f35b60015433146103005760006000fd5b6004357f000000000000000000000000ffffffffffffffffffffffffffffffffffffffff1660007fDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF60206000602435856000526040600020805482018083119155818060025401806002551034171761037f57a3600160005260206000f35b60006000fd";

// aztec huff erc20

const input : &'static str = "
BALANCE_LOCATION = 0

OWNER_LOCATION = 1

TOTAL_SUPPLY_LOCATION = 2

ALLOWANCE_LOCATION = 3

TRANSFER_EVENT_SIGNATURE = 0xDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF

APPROVAL_EVENT_SIGNATURE = 0x8C5BE1E5EBEC7D5BD14F71427D1E84F3DD0314C0F7B2291E5B200AC8C7C3B925

ADDRESS_MASK = () {  0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff and  }

UTILS__NOT_PAYABLE = (error_location) {
	callvalue error_location jumpi
}

UTILS__ONLY_OWNER = () {
	OWNER_LOCATION sload caller eq is_owner jumpi
		0x00 0x00 revert
	is_owner:
}

ERC20 = () {
	caller OWNER_LOCATION sstore
}


ERC20__FUNCTION_SIGNATURE = (transfer, transfer_from, balance_of, allowance, approve, total_supply, mint, error_location) {
	0x00 calldataload 224 shr
	dup1 0x095ea7b3 eq approve jumpi
	dup1 0x18160ddd eq total_supply jumpi
	dup1 0x23b872dd eq transfer_from jumpi
	dup1 0x40c10f19 eq mint jumpi
	dup1 0x70a08231 eq balance_of jumpi
	dup1 0xa9059cbb eq transfer jumpi
	dup1 0xdd62ed3e eq allowance jumpi
	UTILS__NOT_PAYABLE(error_location)
	0x00 0x00 return
}

ERC20__TRANSFER_INIT = () {
	0x04 calldataload ADDRESS_MASK()
	caller
	TRANSFER_EVENT_SIGNATURE
	0x20
	0x00
	0x24 calldataload
	
}

ERC20__TRANSFER_GIVE_TO = () {
	
	dup6 0x00 mstore
	0x40 0x00 sha3
	
	dup1 sload
	
	dup3
	add
	dup1
	dup4
	gt
	swap2
	sstore
}


ERC20__TRANSFER_TAKE_FROM = (from) {
	
	from 0x00 mstore
	0x40 0x00 sha3
	
	dup1 sload

	dup4 dup2
	sub dup5
	swap3
	sstore
	lt
}


ERC20__TRANSFER = (error_location) {
	ERC20__TRANSFER_INIT()
	ERC20__TRANSFER_GIVE_TO()
	ERC20__TRANSFER_TAKE_FROM(caller)
	
	callvalue or or error_location jumpi
	
	0x00 mstore
	log3
	0x01 0x00 mstore
	0x20 0x00 return
}

ERC20__TRANSFER_FROM_INIT = () {
	0x24 calldataload ADDRESS_MASK()
	0x04 calldataload ADDRESS_MASK()
	TRANSFER_EVENT_SIGNATURE
	0x20
	0x00
	0x44 calldataload
	
}

ERC20__TRANSFER_SUB_ALLOWANCE = () {
	
	dup7 0x00 mstore
	ALLOWANCE_LOCATION 0x20 mstore
	0x40 0x00 sha3
	0x20 mstore
	caller 0x00 mstore
	0x40 0x00 sha3
	
	dup1 sload
	dup5 dup2
	sub dup6
	swap3 sstore
	lt
}


ERC20__TRANSFER_FROM = (error_location) {
	ERC20__TRANSFER_FROM_INIT()
	ERC20__TRANSFER_GIVE_TO()
	ERC20__TRANSFER_TAKE_FROM(dup6)
	ERC20__TRANSFER_SUB_ALLOWANCE()
	
	callvalue or or or error_location jumpi
	
	0x00 mstore
	log3
	0x01 0x00 mstore
	0x20 0x00 return
}


ERC20__BALANCE_OF = (error_location) {
	UTILS__NOT_PAYABLE(error_location)
	0x04 calldataload ADDRESS_MASK()
	0x00 mstore
	0x40 0x00 sha3
	sload
	0x00 mstore
	0x20 0x00 return
}


ERC20__ALLOWANCE = (error_location) {
	UTILS__NOT_PAYABLE(error_location)
	0x04 calldataload ADDRESS_MASK()
	0x00 mstore
	ALLOWANCE_LOCATION 0x20 mstore
	0x40 0x00 sha3
	
	0x20 mstore
	0x24 calldataload ADDRESS_MASK()
	0x00 mstore
	0x40 0x00 sha3
	
	sload
	0x00 mstore
	0x20 0x00 return
}


ERC20__APPROVE = (error_location) {
	UTILS__NOT_PAYABLE(error_location)
	0x04 calldataload ADDRESS_MASK()
	caller
	APPROVAL_EVENT_SIGNATURE
	0x20
	0x00
	
	0x24 calldataload
	dup1
	
	caller 0x00 mstore
	ALLOWANCE_LOCATION 0x20 mstore
	0x40 0x00 sha3
	0x20 mstore
	dup7 0x00 mstore
	0x40 0x00 sha3
	
	sstore
	0x00 mstore
	
	log3
	0x01 0x00 mstore
	0x20 0x00 return
}

ERC20__TOTAL_SUPPLY = (error_location) {
	UTILS__NOT_PAYABLE(error_location)
	TOTAL_SUPPLY_LOCATION sload
	0x00 mstore
	0x20 0x00 return
}

ERC20__MINT = (error_location) {
	UTILS__ONLY_OWNER()
	0x04 calldataload ADDRESS_MASK()
	0 TRANSFER_EVENT_SIGNATURE 0x20 0x00
	0x24 calldataload
	
	ERC20__TRANSFER_GIVE_TO()
	
	dup2 dup1
	
	TOTAL_SUPPLY_LOCATION sload add dup1 TOTAL_SUPPLY_LOCATION sstore
	lt
	
	callvalue or or error_location jumpi
	log3
	0x01 0x00 mstore
	0x20 0x00 return
}

MAIN = () {

	ERC20__FUNCTION_SIGNATURE(
		transfer,
		transfer_from,
		balance_of,
		allowance,
		approve,
		total_supply,
		mint,
		throw_error
	)

	transfer:
		ERC20__TRANSFER(throw_error)
	transfer_from:
		ERC20__TRANSFER_FROM(throw_error)
	balance_of:
		ERC20__BALANCE_OF(throw_error)
	allowance:
		ERC20__ALLOWANCE(throw_error)
	approve:
		ERC20__APPROVE(throw_error)
	total_supply:
		ERC20__TOTAL_SUPPLY(throw_error)
	mint:
	    ERC20__MINT(throw_error)

	throw_error:
		0x00 0x00 revert

}

";

#[test]
fn compile_erc20() {
    assert_eq!(must_compile(input), bytecode.to_lowercase());
}
