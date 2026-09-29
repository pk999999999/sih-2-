// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @notice Commitments only. Human identities, filenames and evidence stay off-chain.
contract EvidenceRegistry {
    address public immutable registrar;
    struct Evidence { bytes32 sha256; bytes32 custodian; uint256 registeredAt; }
    struct Custody { bytes32 fromEntity; bytes32 toEntity; uint256 timestamp; }
    mapping(bytes32 => Evidence) private evidence;
    mapping(bytes32 => Custody[]) private history;
    event Registered(bytes32 indexed id, bytes32 sha256);
    event Transferred(bytes32 indexed id, bytes32 fromEntity, bytes32 toEntity);
    constructor() { registrar = msg.sender; }
    modifier authorized() { require(msg.sender == registrar, "registrar only"); _; }
    function registerEvidence(bytes32 id, bytes32 digest, bytes32 custodian) external authorized {
        require(evidence[id].registeredAt == 0, "already registered");
        require(id != bytes32(0) && digest != bytes32(0) && custodian != bytes32(0), "empty commitment");
        evidence[id] = Evidence(digest, custodian, block.timestamp);
        history[id].push(Custody(bytes32(0), custodian, block.timestamp));
        emit Registered(id, digest);
    }
    function getEvidence(bytes32 id) public view returns (Evidence memory) {
        require(evidence[id].registeredAt != 0, "not registered");
        return evidence[id];
    }
    function verifyEvidence(bytes32 id, bytes32 digest) external view returns (bool) {
        return getEvidence(id).sha256 == digest;
    }
    function transferCustody(bytes32 id, bytes32 fromEntity, bytes32 toEntity) external authorized {
        Evidence memory item = getEvidence(id);
        require(item.custodian == fromEntity, "stale custodian");
        require(toEntity != bytes32(0) && toEntity != fromEntity, "invalid recipient");
        evidence[id].custodian = toEntity;
        history[id].push(Custody(fromEntity, toEntity, block.timestamp));
        emit Transferred(id, fromEntity, toEntity);
    }
    function getCustodyHistory(bytes32 id) external view returns (Custody[] memory) {
        getEvidence(id);
        return history[id];
    }
}
