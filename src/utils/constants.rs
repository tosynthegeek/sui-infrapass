pub const CLOCK_OBJECT_ID: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000006";
pub const PACKAGE_ID: &str = "0xc2da3cffefcd735d2d6b702e1dd266e36f6e234fc5eee775f462fc0e8527b379";
pub const REGISTRY_ID: &str = "0x1326718c51b30dd21db59db3a2fdb184a424e16daca0f6717433dd91f0e50553";
pub const ENTITLEMENT_STORE_ID: &str =
    "0x4a4a9fdc8b94f62284fbfe08fdeac3fb399247b734541a97fbcd8866faafb853";
pub const USAGE_RELAYER_ID: &str =
    "0xdb0ff7c0d68095a63e998dd42a139dc948c6f4e80c4d0f5ab7324298bdec837a";

// Tokens
pub const TEST_TOKEN_PACKAGE_ID: &str =
    "0x842f1bc7ec3e93164b3fc28a2b696409d7e41d152c9233cf300bb9a185e5066b";
pub const MAINNET_WAL: &str = "0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59";
pub const MAINNET_USDC: &str = "dba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7";
pub const MAINNET_USDT: &str = "375f70cf2ae4c00bf37117d0c85a2c71545e6ee05c4a5c7d282cd66a4504b068";

pub const TEST_WAL: &str = "0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59";
pub const TEST_USDC: &str = "dba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7";
pub const TEST_USDT: &str = "375f70cf2ae4c00bf37117d0c85a2c71545e6ee05c4a5c7d282cd66a4504b068";

pub const MIGRATIONS_PATH: &str = "src/db/migrations";

pub const LUA_ATOMIC_CHECK_AND_DECREMENT: &str = r#"
    local quota_key = KEYS[1]
    local cost = tonumber(ARGV[1])
    local tier_type = tonumber(ARGV[2])

    if tier_type == 0 then
        return 0
    end

    if tier_type == 2 or tier_type == 3 then
        local current = redis.call('GET', quota_key)
        if current == false then
            return -2 
        end
        current = tonumber(current)
        if current < cost then
            return -1
        end
        return redis.call('DECRBY', quota_key, cost)
    end

    return -3
"#;
