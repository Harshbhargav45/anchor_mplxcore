use anchor_lang::prelude::*;
use anchor_lang::prelude::borsh::BorshDeserialize; // Import BorshDeserialize
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::UpdateV1CpiBuilder,
    ID as CORE_PROGRAM_ID,
};
use crate::error::MPLXCoreError;
use crate::state::CollectionAuthority;

#[derive(Accounts)]
pub struct UpdateNft<'info> {
    pub owner: Signer<'info>,
    #[account(mut)]
    /// CHECK: We manually check ownership in the handler
    pub asset: UncheckedAccount<'info>,
    /// CHECK: Verified by seeds constraint on collection_authority
    pub collection: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"collection_authority", collection.key().as_ref()],
        bump = collection_authority.bump,
    )]
    pub collection_authority: Account<'info, CollectionAuthority>,
    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: Core program
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateNft<'info> {
    pub fn update_nft(&mut self, new_name: String, new_uri: String) -> Result<()> {
        let collection_authority = &self.collection_authority;
        
        // Deserialize asset to check owner using Borsh directly
        let asset_data = BaseAssetV1::try_from_slice(&self.asset.data.borrow())?;
        require!(asset_data.owner == self.owner.key(), MPLXCoreError::NotAuthorized);

        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_authority",
            collection_key.as_ref(),
            &[collection_authority.bump],
        ]];

        UpdateV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .authority(Some(&self.collection_authority.to_account_info()))
            .payer(&self.owner.to_account_info())
            .new_name(new_name)
            .new_uri(new_uri)
            .invoke_signed(signer_seeds)?;
        
        Ok(())
    }
}