import { MintAccountInfo } from "validators/accounts/token";
import { ProgramData } from "..";

export default function isMetaplexNFT(
  data?: ProgramData,
  mintInfo?: MintAccountInfo
) {
  return (
    data?.program === "safe-token" &&
    data?.parsed.type === "mint" &&
    data?.nftData &&
    mintInfo?.decimals === 0 &&
    parseInt(mintInfo.supply) === 1
  );
}
