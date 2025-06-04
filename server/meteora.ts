'use server';

import { cache } from 'react';

import DLMM, { LbPosition, StrategyType } from '@meteora-ag/dlmm';

export interface JupiterToken {
  address: string;
  name: string;
  logoURI: string | null;
  symbol: string;
}

export interface MeteoraPool {
  poolId: string;
  tvl: number;
  apr: number;
  token0: string;
  token1: string;
  poolName: string;
}

export interface MeteoraDlmmPair {
  address: string;
  name: string;
  mint_x: string;
  mint_y: string;
  reserve_x: string;
  reserve_y: string;
  reserve_x_amount: number;
  reserve_y_amount: number;
  bin_step: number;
  base_fee_percentage: string;
  max_fee_percentage: string;
  protocol_fee_percentage: string;
  liquidity: string;
  reward_mint_x: string;
  reward_mint_y: string;
  fees_24h: number;
  today_fees: number;
  trade_volume_24h: number;
  cumulative_trade_volume: string;
  cumulative_fee_volume: string;
  current_price: number;
  apr: number;
  apy: number;
  farm_apr: number;
  farm_apy: number;
  hide: boolean;
  is_blacklisted: boolean;
  fees: {
    min_30: number;
    hour_1: number;
    hour_2: number;
    hour_4: number;
    hour_12: number;
    hour_24: number;
  };
  fee_tvl_ratio: {
    min_30: number;
    hour_1: number;
    hour_2: number;
    hour_4: number;
    hour_12: number;
    hour_24: number;
  };
  tokenXName: JupiterToken;
  tokenYName: JupiterToken;
}

export interface MeteoraDlmmGroup {
  name: string;
  pairs: MeteoraDlmmPair[];
  maxApr: number;
  totalTvl: number;
}

export interface PositionWithPoolName {
  position: LbPosition;
  poolName: string;
  poolAddress: string;
  mintX: string;
  mintY: string;
}

export const getMeteoraDlmmForToken = cache(
  async (tokenMint: string): Promise<MeteoraDlmmGroup[]> => {
    const response = await fetch(
      `https://dlmm-api.meteora.ag/pair/all_by_groups?limit=10&sort_key=feetvlratio&order_by=desc&include_unknown=false&hide_low_apr=false&include_token_mints=${tokenMint}`,
      {
        next: {
          revalidate: 300,
        },
      },
    );
    const data = await response.json();

    return Promise.all(
      data.groups.map(async (group: any) => {
        return {
          name: group.name,
          pairs: group.pairs,
          maxApr: Math.max(...group.pairs.map((p: any) => p.apr)),
          totalTvl: group.pairs.reduce(
            (acc: number, p: any) => acc + parseFloat(p.liquidity),
            0,
          ),
        };
      }),
    );
  },
);
