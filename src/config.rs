use regex::{Captures, Regex};
use std::collections::{HashMap, HashSet};

use crate::util;

pub fn generate_name(text: &str) -> String {
    lazy_static! {
        // regex for dashable chars
        static ref DASHABLE_REG: Regex = Regex::new(r"[\s/(,]+").unwrap();
        // regex for useless chars
        static ref USELESS_REG: Regex = Regex::new(r"[^a-z0-9_-]+").unwrap();
    }

    let mut text = text.to_lowercase();
    text = text.replace("()", "");
    text = util::regex::replace_all(&DASHABLE_REG, &text, |_: &Captures| "-".to_owned());
    text = util::regex::replace_all(&USELESS_REG, &text, |_: &Captures| "".to_owned());
    text
}

// Generate a safe group name from a key.
pub fn generate_group_name(key: &str) -> String {
    let safe_chars = "abcdefghijklmnopqrstuvwxyz0123456789";
    let group_length = 2;

    let mut group = String::with_capacity(group_length);

    for ch in key.to_lowercase().chars() {
        if group.len() == group_length {
            break;
        }

        if safe_chars.contains(ch) {
            group.push(ch);
        }
    }

    if group.len() < group_length {
        group.push_str(&"_".repeat(group_length - group.len()));
    }

    group
}

lazy_static! {
    pub static ref SOURCE_FILE_EXTENSIONS: HashSet<&'static str> = {
        hashset! {".bs", ".src.html"}
    };
    pub static ref INLINE_ELEMENT_TAGS: HashSet<&'static str> = {
        hashset! {
            "a", "em", "strong", "small", "s", "cite", "q", "dfn", "abbr", "data", "time", "code",
            "var", "samp", "kbd", "sub", "sup", "i", "b", "u", "mark", "ruby", "bdi", "bdo", "span",
            "br", "wbr", "img", "meter", "progress", "css", "l",
        }
    };
    pub static ref DFN_CLASS_TO_TYPE: HashMap<&'static str, &'static str> = {
        hashmap! {
            "propdef" => "property",
            "descdef" => "descriptor",
        }
    };
    pub static ref DFN_TYPE_TO_CLASS: HashMap<&'static str, &'static str> = {
        let mut dfn_type_to_class: HashMap<&'static str, &'static str> = HashMap::new();

        for (key, val) in DFN_CLASS_TO_TYPE.iter() {
            dfn_type_to_class.insert(val, key);
        }

        dfn_type_to_class
    };
    pub static ref DFN_TYPES: HashSet<&'static str> = {
        let mut dfn_types: HashSet<&'static str> = hashset! {"dfn"};

        for val in DFN_CLASS_TO_TYPE.values() {
            dfn_types.insert(val);
        }

        dfn_types
    };
    pub static ref LINK_TYPES: HashSet<&'static str> = { DFN_TYPES.clone() };
    pub static ref SHORT_TO_LONG_STATUS: HashMap<&'static str, &'static str> = {
        hashmap! {
            "DREAM" => "A Collection of Interesting Ideas",
            "LS" => "Living Standard",
            "LS-COMMIT" => "Commit Snapshot",
            "LS-BRANCH" => "Branch Snapshot",
            "LD" => "Living Document",
            "DRAFT-FINDING" => "Draft Finding",
            "FINDING" => "Finding",
            "whatwg/RD" => "Review Draft",
            "w3c/ED" => "Editor's Draft",
            "w3c/WD" => "W3C Working Draft",
            "w3c/FPWD" => "W3C First Public Working Draft",
            "w3c/LCWD" => "W3C Last Call Working Draft",
            "w3c/CR" => "W3C Candidate Recommendation",
            "w3c/PR" => "W3C Proposed Recommendation",
            "w3c/REC" => "W3C Recommendation",
            "w3c/PER" => "W3C Proposed Edited Recommendation",
            "w3c/WG-NOTE" => "W3C Working Group Note",
            "w3c/IG-NOTE" => "W3C Interest Group Note",
            "w3c/NOTE" => "W3C Note",
            "w3c/MO" => "W3C Member-only Draft",
            "w3c/UD" => "Unofficial Proposal Draft",
            "w3c/CG-DRAFT" => "Draft Community Group Report",
            "w3c/CG-FINAL" => "Final Community Group Report",
            "tc39/STAGE0" => "Stage 0: Strawman",
            "tc39/STAGE1" => "Stage 1: Proposal",
            "tc39/STAGE2" => "Stage 2: Draft",
            "tc39/STAGE3" => "Stage 3: Candidate",
            "tc39/STAGE4" => "Stage 4: Finished",
            "iso/I" => "Issue",
            "iso/DR" =>"Defect Report",
            "iso/D" => "Draft Proposal",
            "iso/P" => "Published Proposal",
            "iso/MEET" => "Meeting Announcements",
            "iso/RESP" => "Records of Response",
            "iso/MIN" => "Minutes",
            "iso/ER" => "Editor's Report",
            "iso/SD" => "Standing Document",
            "iso/PWI" => "Preliminary Work Item",
            "iso/NP" => "New Proposal",
            "iso/NWIP" => "New Work Item Proposal",
            "iso/WD" => "Working Draft",
            "iso/CD" => "Committee Draft",
            "iso/FCD" => "Final Committee Draft",
            "iso/DIS" => "Draft International Standard",
            "iso/FDIS" => "Final Draft International Standard",
            "iso/PRF" => "Proof of a new International Standard",
            "iso/IS" => "International Standard",
            "iso/TR" => "Technical Report",
            "iso/DTR" => "Draft Technical Report",
            "iso/TS" => "Technical Specification",
            "iso/DTS" => "Draft Technical Specification",
            "iso/PAS" => "Publicly Available Specification",
            "iso/TTA" => "Technology Trends Assessment",
            "iso/IWA" => "International Workshop Agreement",
            "iso/COR" => "Technical Corrigendum",
            "iso/GUIDE" => "Guidance to Technical Committees",
            "iso/NP-AMD" => "New Proposal Amendment",
            "iso/AWI-AMD" => "Approved new Work Item Amendment",
            "iso/WD-AMD" => "Working Draft Amendment",
            "iso/CD-AMD" => "Committee Draft Amendment",
            "iso/PD-AMD" => "Proposed Draft Amendment",
            "iso/FPD-AMD" => "Final Proposed Draft Amendment",
            "iso/D-AMD" => "Draft Amendment",
            "iso/FD-AMD" => "Final Draft Amendment",
            "iso/PRF-AMD" => "Proof Amendment",
            "iso/AMD" => "Amendment",
            "fido/ED" => "Editor's Draft",
            "fido/WD" => "Working Draft",
            "fido/RD" => "Review Draft",
            "fido/ID" => "Implementation Draft",
            "fido/PS" => "Proposed Standard",
            "fido/FD" => "Final Document",
            "khronos/ED" => "Editor's Draft"
        }
    };
}
