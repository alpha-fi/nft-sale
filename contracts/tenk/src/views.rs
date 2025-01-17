use crate::*;

#[near_bindgen]
impl Contract {
    /// Current contract owner
    pub fn owner(&self) -> AccountId {
        self.tokens.owner_id.clone()
    }

    /// Current set of admins
    pub fn admins(&self) -> Vec<AccountId> {
        self.admins.to_vec()
    }

    /// Check whether an account is allowed to mint during the presale
    pub fn whitelisted(&self, account_id: &AccountId) -> bool {
        self.whitelist.contains_key(account_id)
    }

    /*
        /// Cost of NFT + fees for linkdrop
        pub fn cost_of_linkdrop(&self, minter: &AccountId) -> U128 {
            (self.full_link_price(minter)
                + self.total_cost(1, minter, false).0
                + self.token_storage_cost().0)
                .into()
        }
    */

    pub fn total_cost(&self, num: u32, minter: &AccountId, with_cheddar: bool) -> U128 {
        let mut cost = self.minting_cost(minter, num).0;
        if with_cheddar {
            cost = cost / 1000 * self.cheddar_near / 100 * self.cheddar_boost as u128;
        }
        cost.into()
    }

    /// Flat cost in NEAR for minting given amount of tokens
    pub fn minting_cost(&self, minter: &AccountId, num: u32) -> U128 {
        if self.is_owner(minter) {
            0
        } else {
            self.price(num)
        }
        .into()
    }

    /// Current cost in NEAR to store one NFT
    pub fn token_storage_cost(&self) -> U128 {
        (env::storage_byte_cost() * self.tokens.extra_storage_in_bytes_per_token as Balance).into()
    }

    /// Tokens left to be minted.  This includes those left to be raffled minus any pending linkdrops
    pub fn tokens_left(&self) -> u32 {
        self.raffle.len() as u32 - self.pending_tokens
    }

    /// Part of the NFT metadata standard. Returns the contract's metadata
    pub fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }

    /// How many tokens an account is still allowed to mint. None, means unlimited
    pub fn remaining_allowance(&self, account_id: &AccountId) -> Option<u32> {
        self.whitelist.get(account_id)
    }

    /// Max number of mints in one transaction. None, means unlimited
    pub fn mint_rate_limit(&self) -> Option<u32> {
        self.sale.mint_rate_limit
    }

    /// Information about the current sale. When in starts, status, price, and how many could be minted.
    pub fn get_sale_info(&self) -> SaleInfo {
        SaleInfo {
            presale_start: self.sale.presale_start.unwrap_or(MAX_DATE),
            sale_start: self.sale.public_sale_start.unwrap_or(MAX_DATE),
            status: self.get_status(),
            price: self.price(1).into(),
            token_final_supply: self.initial(),
            tokens_sold: self.counter,
        }
    }

    /// Information about a current user. Whether they are VIP and how many tokens left in their allowance.
    pub fn get_user_sale_info(&self, account_id: &AccountId) -> UserSaleInfo {
        let sale_info = self.get_sale_info();
        let remaining_allowance = if self.is_presale() || self.sale.allowance.is_some() {
            self.remaining_allowance(account_id)
        } else {
            None
        };
        UserSaleInfo {
            sale_info,
            remaining_allowance,
            is_vip: self.whitelisted(account_id),
        }
    }

    /// Initial size of collection. Number left to raffle + current total supply
    pub fn initial(&self) -> u64 {
        self.raffle.len() + self.nft_total_supply().0 as u64
    }
}
