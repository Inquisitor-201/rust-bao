// UART Base Address (PL011)
const UART_BASE_0: u32 = 0xFDF02000;
const UART_BASE_1: u32 = 0xFDF00000;
const UART_BASE_2: u32 = 0xFDF03000;
const UART_BASE_4: u32 = 0xFDF01000;
const UART_BASE_5: u32 = 0xFDF05000;
const UART_BASE_6: u32 = 0xFFF32000;

// UART Interrupts
const UART_0_INTERRUPT: u32 = 106;
const UART_1_INTERRUPT: u32 = 107;
const UART_2_INTERRUPT: u32 = 108;
const UART_4_INTERRUPT: u32 = 109;
const UART_5_INTERRUPT: u32 = 110;
const UART_6_INTERRUPT: u32 = 111;

const NUM_UART: u32 = 6;

const UART_CLK: u32 = 19200000;
const UART_BAUD_RATE: u32 = 115200;

// UART Data Register
const UART_DATA_DATA: u32 = 0xFFFFFF00;
const UART_DATA_FE: u32 = 1 << 8;
const UART_DATA_PE: u32 = 1 << 9;
const UART_DATA_BE: u32 = 1 << 10;
const UART_DATA_OE: u32 = 1 << 11;

// UART Receive Status Register/Error Clear Register
const UART_RSR_ECR_FE: u32 = 1 << 0;
const UART_RSR_ECR_PE: u32 = 1 << 1;
const UART_RSR_ECR_BE: u32 = 1 << 2;
const UART_RSR_ECR_OE: u32 = 1 << 3;
const UART_RSR_ECR_CLEAR: u32 = 0xFFFFFF00;

// UART Flag Register
const UART_FR_CTS: u32 = 1 << 0;
const UART_FR_DSR: u32 = 1 << 1;
const UART_FR_DCD: u32 = 1 << 2;
const UART_FR_BUSY: u32 = 1 << 3;
const UART_FR_RXFE: u32 = 1 << 4;
const UART_FR_TXFF: u32 = 1 << 5;
const UART_FR_RXFF: u32 = 1 << 6;
const UART_FR_TXFE: u32 = 1 << 7;
const UART_FR_RI: u32 = 1 << 8;

// UART Integer Baud Rate Register
const UART_IBRD_DIVINT: u32 = 0x0000FFFF;

// UART Fractional Baud Rate Register
const UART_FBRD_DIVFRAC: u32 = 0x0000003F;

// UART Line Control Register
const UART_LCR_BRK: u32 = 1 << 0;
const UART_LCR_PEN: u32 = 1 << 1;
const UART_LCR_EPS: u32 = 1 << 2;
const UART_LCR_STP2: u32 = 1 << 3;
const UART_LCR_FEN: u32 = 1 << 4;
const UART_LCR_WLEN_8: u32 = 0b11 << 5;
const UART_LCR_WLEN_7: u32 = 0b10 << 5;
const UART_LCR_WLEN_6: u32 = 0b01 << 5;
const UART_LCR_WLEN_5: u32 = 0b00 << 5;
const UART_LCR_SPS: u32 = 1 << 7;

// UART Control Register
const UART_CR_UARTEN: u32 = 1 << 0;
const UART_CR_SIREN: u32 = 1 << 1;
const UART_CR_SIRLP: u32 = 1 << 2;
const UART_CR_LBE: u32 = 1 << 7;
const UART_CR_TXE: u32 = 1 << 8;
const UART_CR_RXE: u32 = 1 << 9;
const UART_CR_DTR: u32 = 1 << 10;
const UART_CR_RTS: u32 = 1 << 11;
const UART_CR_OUT1: u32 = 1 << 12;
const UART_CR_OUT2: u32 = 1 << 13;
const UART_CR_RTSE: u32 = 1 << 14;
const UART_CR_CTSE: u32 = 1 << 15;

// UART Interrupt FIFO Level Select Register
const UART_IFLS_TXIFLSEL_1_8: u32 = 0b000 << 0;
const UART_IFLS_TXIFLSEL_1_4: u32 = 0b001 << 0;
const UART_IFLS_TXIFLSEL_1_2: u32 = 0b010 << 0;
const UART_IFLS_TXIFLSEL_3_4: u32 = 0b011 << 0;
const UART_IFLS_TXIFLSEL_7_8: u32 = 0b100 << 0;
const UART_IFLS_RXIFLSEL_1_8: u32 = 0b000 << 3;
const UART_IFLS_RXIFLSEL_1_4: u32 = 0b001 << 3;
const UART_IFLS_RXIFLSEL_1_2: u32 = 0b010 << 3;
const UART_IFLS_RXIFLSEL_3_4: u32 = 0b011 << 3;
const UART_IFLS_RXIFLSEL_7_8: u32 = 0b100 << 3;

// UART Interrupt Mask Set/Clear Register
const UART_IMSC_RIMIM: u32 = 1 << 0;
const UART_IMSC_CTSMIM: u32 = 1 << 1;
const UART_IMSC_DCDMIM: u32 = 1 << 2;
const UART_IMSC_DSRMI: u32 = 1 << 3;
const UART_IMSC_RXIM: u32 = 1 << 4;
const UART_IMSC_TXIM: u32 = 1 << 5;
const UART_IMSC_RTIM: u32 = 1 << 6;
const UART_IMSC_FEIM: u32 = 1 << 7;
const UART_IMSC_PEIM: u32 = 1 << 8;
const UART_IMSC_BEIM: u32 = 1 << 9;
const UART_IMSC_OEIM: u32 = 1 << 10;

// UART Raw Interrupt Status Register
const UART_RIS_RIRMIS: u32 = 1 << 0;
const UART_RIS_CTSRMIS: u32 = 1 << 1;
const UART_RIS_DCDRMIS: u32 = 1 << 2;
const UART_RIS_DSRRMIS: u32 = 1 << 3;
const UART_RIS_RXRIS: u32 = 1 << 4;
const UART_RIS_TXRIS: u32 = 1 << 5;
const UART_RIS_RTRIS: u32 = 1 << 6;
const UART_RIS_FERIS: u32 = 1 << 7;
const UART_RIS_PERIS: u32 = 1 << 8;
const UART_RIS_BERIS: u32 = 1 << 9;
const UART_RIS_OERIS: u32 = 1 << 10;

// UART Masked Interrupt Status Register
const UART_MIS_RIMMIS: u32 = 1 << 0;
const UART_MIS_CTSMMIS: u32 = 1 << 1;
const UART_MIS_DCDMMIS: u32 = 1 << 2;
const UART_MIS_DSRMMIS: u32 = 1 << 3;
const UART_MIS_RXMIS: u32 = 1 << 4;
const UART_MIS_TXMIS: u32 = 1 << 5;
const UART_MIS_RTMIS: u32 = 1 << 6;
const UART_MIS_FEMIS: u32 = 1 << 7;
const UART_MIS_PEMIS: u32 = 1 << 8;
const UART_MIS_BEMIS: u32 = 1 << 9;
const UART_MIS_OEMIS: u32 = 1 << 10;

// UART Interrupt Clear Register
const UART_ICR_RIMIC: u32 = 1 << 0;
const UART_ICR_CTSMIC: u32 = 1 << 1;
const UART_ICR_DCDMIC: u32 = 1 << 2;
const UART_ICR_DSRMIC: u32 = 1 << 3;
const UART_ICR_RXIC: u32 = 1 << 4;
const UART_ICR_TXIC: u32 = 1 << 5;
const UART_ICR_RTIC: u32 = 1 << 6;
const UART_ICR_FEIC: u32 = 1 << 7;
const UART_ICR_PEIC: u32 = 1 << 8;
const UART_ICR_BEIC: u32 = 1 << 9;
const UART_ICR_OEIC: u32 = 1 << 10;

// UART DMA Control Register
const UART_DMACR_RXDMAE: u32 = 1 << 0;
const UART_DMACR_TXDMAE: u32 = 1 << 1;
const UART_DMACR_DMAONERR: u32 = 1 << 2;

#[repr(C)]
pub struct Pl011UartHW {
    pub data: u32,               // UART Data Register
    pub status_error: u32,       // UART Receive Status Register/Error Clear Register
    reserved1: [u32; 4],         // Reserved: 4(0x4) bytes
    pub flag: u32,               // UART Flag Register
    reserved2: u32,              // Reserved: 1(0x1) bytes
    pub lp_counter: u32,         // UART Low-power Counter Register
    pub integer_br: u32,         // UART Integer Baud Rate Register
    pub fractional_br: u32,      // UART Fractional Baud Rate Register
    pub line_control: u32,       // UART Line Control Register
    pub control: u32,            // UART Control Register
    pub isr_fifo_level_sel: u32, // UART Interrupt FIFO level Select Register
    pub isr_mask: u32,           // UART Interrupt Mask Set/Clear Register
    pub raw_isr_status: u32,     // UART Raw Interrupt Status Register
    pub masked_isr_status: u32,  // UART Masked Interrupt Status Register
    pub isr_clear: u32,          // UART Interrupt Clear Register
    pub dma_control: u32,        // UART DMA control Register
}

impl Pl011UartHW {
    pub fn disable(&mut self) {
        let mut ctrl_reg = self.control;
        ctrl_reg &= !UART_CR_UARTEN & !UART_CR_TXE & !UART_CR_RXE;
        self.control = ctrl_reg;
    }

    pub fn enable(&mut self) {
        let mut ctrl_reg = self.control;
        ctrl_reg |= UART_CR_UARTEN | UART_CR_TXE | UART_CR_RXE;
        self.control = ctrl_reg;
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        let temp: u32;
        let ibrd: u32;
        let mod_value: u32;
        let fbrd: u32;

        let clk = UART_CLK;
        let default_baud_rate = UART_BAUD_RATE;
        let baud_rate = if baud_rate == 0 {
            default_baud_rate
        } else {
            baud_rate
        };

        // Set baud rate
        temp = 16 * baud_rate;
        ibrd = clk / temp;
        mod_value = clk % temp;
        fbrd = (4 * mod_value) / baud_rate;

        // Set the values of the baudrate divisors
        self.integer_br = ibrd;
        self.fractional_br = fbrd;
    }

    pub fn init(&mut self) {
        let mut lcrh_reg: u32;

        // First, disable everything
        self.control = 0x0;

        // Disable FIFOs
        lcrh_reg = self.line_control;
        lcrh_reg &= !UART_LCR_FEN;
        self.line_control = lcrh_reg;

        // Default baudrate = 115200
        let baud_rate = UART_BAUD_RATE;
        self.set_baud_rate(baud_rate);

        // Set the UART to be 8 bits, 1 stop bit and no parity, FIFOs enable
        self.line_control = UART_LCR_WLEN_8 | UART_LCR_FEN;

        // Enable the UART, enable TX and enable loop back
        self.control = UART_CR_UARTEN | UART_CR_TXE | UART_CR_LBE;

        // Set the receive interrupt FIFO level to 1/2 full
        self.isr_fifo_level_sel = UART_IFLS_RXIFLSEL_1_2;

        while self.flag & UART_FR_BUSY != 0 {}

        // Enable RX
        self.control = UART_CR_UARTEN | UART_CR_RXE | UART_CR_TXE;

        // Clear interrupts
        self.isr_clear = UART_ICR_OEIC | UART_ICR_BEIC | UART_ICR_PEIC | UART_ICR_FEIC;

        // Enable receive and receive timeout interrupts
        self.isr_mask = UART_MIS_RXMIS | UART_MIS_RTMIS;
    }

    pub fn getc(&mut self) -> u32 {
        // Wait until there is data in FIFO
        while self.flag & UART_FR_RXFE != 0 {}

        self.data
    }

    pub fn putc(&mut self, c: i8) {
        // Wait until txFIFO is not full
        while self.flag & UART_FR_TXFF != 0 {}

        self.data = c as u32;
    }

    pub fn puts(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c as i8);
        }
    }
}
