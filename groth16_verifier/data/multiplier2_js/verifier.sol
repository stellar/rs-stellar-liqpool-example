// SPDX-License-Identifier: GPL-3.0
/*
    Copyright 2021 0KIMS association.

    This file is generated with [snarkJS](https://github.com/iden3/snarkjs).

    snarkJS is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    snarkJS is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with snarkJS. If not, see <https://www.gnu.org/licenses/>.
*/

pragma solidity >=0.7.0 <0.9.0;

contract Groth16Verifier {
    // Scalar field size
    uint256 constant r    = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    // Base field size
    uint256 constant q   = 21888242871839275222246405745257275088696311157297823662689037894645226208583;

    // Verification Key data
    uint256 constant alphax  = 851850525556173310373115880154698084608631105506432893865500290442025919078535925294035153152030470398262539759609;
    uint256 constant alphay  = 2637289349983507610125993281171282870664683328789064436670091381805667870657250691837988574635646688089951719927247;
    uint256 constant betax1  = 1853421227732662200477195678252233549930451033531229987959164216695698667330234953033341200627605777603511819497457;
    uint256 constant betax2  = 1312620381151154625549413690218290437739613987001512553647554932245743783919690104921577716179019375920325686841943;
    uint256 constant betay1  = 812366606879346135498483310623227330050424196838294715759414425317592599094348477520229174120664109186562798527696;
    uint256 constant betay2  = 3215807833988244618006117550809420301978856703407297742347804415291049013404133666905173282837707341742014140541018;
    uint256 constant gammax1 = 3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758;
    uint256 constant gammax2 = 352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160;
    uint256 constant gammay1 = 927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582;
    uint256 constant gammay2 = 1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905;
    uint256 constant deltax1 = 2236695112259305382987038341098587500598216646308901956168137697892380899086228863246537938263638056666003066263342;
    uint256 constant deltax2 = 2981843938988033214458466658185878126396080429969635248100956025957789319926032198626745120548947333202362392267114;
    uint256 constant deltay1 = 3496058064578305387608803828034117220735807855182872031001942587835768203820179263722136810383631418598310938506798;
    uint256 constant deltay2 = 717163810166643254871951856655865822196000925757284470845197358532703820821048809982340614428800986999944933231635;

    
    uint256 constant IC0x = 829685638389803071404995253486571779300247099942205634643821309129201420207693030476756893332812706176564514055395;
    uint256 constant IC0y = 3455508165409829148751617737772894557887792278044850553785496869183933597103951941805834639972489587640583544390358;
    
    uint256 constant IC1x = 2645559270376031734407122278942646687260452979296081924477586893972449945444985371392950465676350735694002713633589;
    uint256 constant IC1y = 2241039659097418315097403108596818813895651201896886552939297756980670248638746432560267634304593609165964274111037;
    
 
    // Memory data
    uint16 constant pVk = 0;
    uint16 constant pPairing = 128;

    uint16 constant pLastMem = 896;

    function verifyProof(uint[2] calldata _pA, uint[2][2] calldata _pB, uint[2] calldata _pC, uint[1] calldata _pubSignals) public view returns (bool) {
        assembly {
            function checkField(v) {
                if iszero(lt(v, r)) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }
            
            // G1 function to multiply a G1 value(x,y) to value in an address
            function g1_mulAccC(pR, x, y, s) {
                let success
                let mIn := mload(0x40)
                mstore(mIn, x)
                mstore(add(mIn, 32), y)
                mstore(add(mIn, 64), s)

                success := staticcall(sub(gas(), 2000), 7, mIn, 96, mIn, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }

                mstore(add(mIn, 64), mload(pR))
                mstore(add(mIn, 96), mload(add(pR, 32)))

                success := staticcall(sub(gas(), 2000), 6, mIn, 128, pR, 64)

                if iszero(success) {
                    mstore(0, 0)
                    return(0, 0x20)
                }
            }

            function checkPairing(pA, pB, pC, pubSignals, pMem) -> isOk {
                let _pPairing := add(pMem, pPairing)
                let _pVk := add(pMem, pVk)

                mstore(_pVk, IC0x)
                mstore(add(_pVk, 32), IC0y)

                // Compute the linear combination vk_x
                
                g1_mulAccC(_pVk, IC1x, IC1y, calldataload(add(pubSignals, 0)))
                

                // -A
                mstore(_pPairing, calldataload(pA))
                mstore(add(_pPairing, 32), mod(sub(q, calldataload(add(pA, 32))), q))

                // B
                mstore(add(_pPairing, 64), calldataload(pB))
                mstore(add(_pPairing, 96), calldataload(add(pB, 32)))
                mstore(add(_pPairing, 128), calldataload(add(pB, 64)))
                mstore(add(_pPairing, 160), calldataload(add(pB, 96)))

                // alpha1
                mstore(add(_pPairing, 192), alphax)
                mstore(add(_pPairing, 224), alphay)

                // beta2
                mstore(add(_pPairing, 256), betax1)
                mstore(add(_pPairing, 288), betax2)
                mstore(add(_pPairing, 320), betay1)
                mstore(add(_pPairing, 352), betay2)

                // vk_x
                mstore(add(_pPairing, 384), mload(add(pMem, pVk)))
                mstore(add(_pPairing, 416), mload(add(pMem, add(pVk, 32))))


                // gamma2
                mstore(add(_pPairing, 448), gammax1)
                mstore(add(_pPairing, 480), gammax2)
                mstore(add(_pPairing, 512), gammay1)
                mstore(add(_pPairing, 544), gammay2)

                // C
                mstore(add(_pPairing, 576), calldataload(pC))
                mstore(add(_pPairing, 608), calldataload(add(pC, 32)))

                // delta2
                mstore(add(_pPairing, 640), deltax1)
                mstore(add(_pPairing, 672), deltax2)
                mstore(add(_pPairing, 704), deltay1)
                mstore(add(_pPairing, 736), deltay2)


                let success := staticcall(sub(gas(), 2000), 8, _pPairing, 768, _pPairing, 0x20)

                isOk := and(success, mload(_pPairing))
            }

            let pMem := mload(0x40)
            mstore(0x40, add(pMem, pLastMem))

            // Validate that all evaluations ∈ F
            
            checkField(calldataload(add(_pubSignals, 0)))
            

            // Validate all evaluations
            let isValid := checkPairing(_pA, _pB, _pC, _pubSignals, pMem)

            mstore(0, isValid)
             return(0, 0x20)
         }
     }
 }
