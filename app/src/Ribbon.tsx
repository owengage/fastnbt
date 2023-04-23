import { ReactFragment } from "react";
import "./Ribbon.css";

export interface RinnonPropsInner {}
type RibbonProps = React.PropsWithChildren<RinnonPropsInner>;

export function Ribbon({ children }: RibbonProps) {
  return <div className="ribbon">{children}</div>;
}
