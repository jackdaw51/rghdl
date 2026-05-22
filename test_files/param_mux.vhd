library ieee;
use ieee.std_logic_1164.all;
use work.custom_types_pkg.all;

entity param_mux is
    generic (
        WIDTH : integer := DEFAULT_BUS_WIDTH
    );
    port (
        sel : in std_logic;
        input0 : in std_logic_vector(WIDTH - 1 downto 0);
        input1 : in std_logic_vector(WIDTH - 1 downto 0);
        output : out std_logic_vector(WIDTH - 1 downto 0)
    );
end entity param_mux;

architecture behavioral of param_mux is
begin
    output <= input0 when sel = '0' else
        input1;
end architecture behavioral;