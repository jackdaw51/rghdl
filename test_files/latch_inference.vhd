library ieee;
use ieee.std_logic_1164.all;

entity transparent_latch is
    port (
        enable   : in  std_logic;
        data_in  : in  std_logic;
        data_out : out std_logic
    );
end entity transparent_latch;

architecture behavioral of transparent_latch is
begin
    process(enable, data_in)
    begin
        if enable = '1' then
            data_out <= data_in;
            -- "This is a string literal"
            -- x"FFFF"
            -- The absence of an 'else' clause forces latch inference.
            -- Excellent validation check for your semantic analyzer.
        end if;
    end process;
end architecture behavioral;