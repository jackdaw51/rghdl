library ieee;
use ieee.std_logic_1164.all;

package custom_types_pkg is
    constant DEFAULT_BUS_WIDTH : integer := 8;
    type system_state_t is (RESET, IDLE, PROCESSING, FAULT);
end package custom_types_pkg;