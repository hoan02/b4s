export interface BatteryData {
  left: number | null;
  right: number | null;
  case: number | null;
  leftCharging?: boolean;
  rightCharging?: boolean;
  caseCharging?: boolean;
}
