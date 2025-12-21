/*
 * da7280_hal.h - DA7280 Haptic Hardware Abstraction Layer Header
 *
 * Copyright (C) 2017 Dialog Semiconductor Ltd. and its Affiliates, unpublished
 * work. This computer program includes Confidential, Proprietary Information
 * and is a Trade Secret of Dialog Semiconductor Ltd. and its Affiliates.
 * All use, disclosure, and/or reproduction is prohibited unless authorized
 * in writing. All Rights Reserved.
 */

#ifndef _DA7280_HAL_H
#define _DA7280_HAL_H

#include "da7280.h"

/* Check types or define it */
#ifndef u8
typedef unsigned char	u8;
typedef unsigned short	u16;
typedef unsigned int	u32;
#endif

#undef DIALOG_TEST_PLATFORM
#define DIALOG_TEST_PLATFORM

#ifdef DIALOG_TEST_PLATFORM
/* To use i2c functions from Dialog TEST Platform */
#include "common-i2c-routines.h"
#else
/* Customer may need to add i2c related head file */
#endif

#ifdef DIALOG_TEST_PLATFORM
/* Debug/Troubleshooting */
#define DA7280_DEBUG

/* Delay Macro Definition */
#define DA7280_SET_DELAY
#else
#undef DA7280_DEBUG
#endif

/* Debug/Troubleshooting -
** have to be replaced to the code platform provide
*/
#ifdef DA7280_DEBUG
#define diag_info printf
#define diag_i2c printf
#define diag_err printf
#define diag_warn printf
#define diag_i2c_read(fmt, ...) do { } while (0)
#else
#define diag_info(fmt, ...) do { } while (0)
#define diag_err(fmt, ...)  do { } while (0)
#define diag_info(fmt, ...) do { } while (0)
#define diag_i2c(fmt, ...) do { } while (0)
#endif

/* Delay macros -
** have to be replaced to the code platform provide
*/
#ifdef DA7280_SET_DELAY
#define mdelay(v)	msleep((v))
#define udelay(v)	usleep((v))
#else
#define mdelay(v)
#define udelay(v)
#endif

/*
* Constant Definitions
*/

/* 7 Bit */
#define DA7280_I2C_SLAVE_ADDR (0x94 >> 1)

#define EINVAL	1
#ifndef EACCES
#define EACCES	13
#endif
#define EBUSY		16
#define EIO	2
#define EOPNOTSUPP	95
#define SEQ_END	0xFFFF
#define DA7280_DELAY 0xFFFE

#define DA7280_VOLT_RATE_MAX		6000
#define DA7280_VOLT_STEP_uV		23400
#define DA7280_NOM_VOLT_DFT		0x6B

#define DA7280_IMAX_STEP		7200

#define DA7280_MIN_RESONAT_FREQ		50
#define DA7280_MAX_RESONAT_FREQ	300

#define DA7280_MIN_PWM_FREQ_kHz	10000
#define DA7280_MAX_PWM_FREQ_kHz	250000

/* Impedance Range, milliOhm unit */
#define DA7280_IMPD_MAX	50000
#define DA7280_IMPD_MIN	4000
#define DA7280_IMAX_LIMIT	252

#define DA7280_SNP_MEM_SIZE		100
#define DA7280_SNP_MEM_MAX DA7280_SNP_MEM_99

#define IRQ_NUM	3

/*
 * DA7280 related types
*/

/*script type */
struct scr_type {
	u16	reg;
	u8	val;
};

struct scr_mask_type {
	u16	reg;
	u8	mask;
	u8	val;
};

enum da7280_haptic_dev_t {
	DA7280_LRA = 0,
	DA7280_ERM_BAR = 1,
	DA7280_ERM_COIN = 2,
	DA7280_DEV_MAX = 3,
};
#define DA7280_DEV_NONE 0xFF

enum da7280_op_mode {
	DA7280_INACTIVE_MODE = 0,
	DA7280_DRO_MODE	= 1,
	DA7280_PWM_MODE	= 2,
	DA7280_RTWM_MODE	= 3,
	DA7280_ETWM_MODE	= 4,
	DA7280_MODE_MAX,
};
#define DA7280_OPMODE_NONE 0xFF

enum da7280_gpi_mod {
	DA7280_SINGLE_PTN	= 0,
	DA7280_MULTI_PTN	= 1,
	DA7280_GPI_MOD_MAX,
};

enum da7280_gpi_pol {
	DA7280_RISING_EDGE		= 0,
	DA7280_FALLING_EDGE	= 1,
	DA7280_BOTH_EDGE		= 2,
	DA7280_GPI_POL_MAX,
};

struct da7280_gpi_ctl {
	u8 seq_id;
	u8 mode;
	u8 polarity;
};

struct da7280_haptic {
	u8 suspend_state;
	unsigned int magnitude;
	u8 dev_type;
	u8 op_mode;
	u8 bemf_sense_en;
	u8 freq_track_en;
	u8 acc_en;
	u8 rapid_stop_en;
	u8 amp_pid_en;
};

/*
 * Customer Platform Definitions and Data start :
 * These definitons should be represented for customer's system.
 */

/* Required User data */
#define USER_HAPTIC_DEV		DA7280_LRA
#define USER_OP_MODE		DA7280_DRO_MODE
#define USER_BEMF_SENS_EN	1
#define USER_FREQ_TRACK_EN	1
#define USER_ACC_EN			1
#define USER_RAPID_STOP_EN	1
#define USER_AMP_PID_EN		0

/* Voltage setting / Unit: mili Volt */
#define USER_NOM_mVolt		1200
#define USER_ABS_MAX_mV		1400

#define USER_RESONANT_FREQ_Hz	180
#define USER_IMAX_mA			137

/* milliOhm unit */
#define USER_IMPD_mOhm		(10500)
#define USER_OVERIDE_VAL		0x59

#define USE_SEQ_ID	7
#define USE_SEQ_LOOP	3

/* To use USER GPI Definitions */
#define DA7280_USER_GPIO
/* SEQ_ID should not be bigger than 15 (0 <= X <= 15) */
#define USER_SEQ_ID_MAX		15
#define USER_SEQ_LOOP_MAX	15
#define USER_GPI_0_SEQ_ID	7
#define USER_GPI_1_SEQ_ID	7
#define USER_GPI_2_SEQ_ID	7

#define USER_GPI_0_MOD	DA7280_SINGLE_PTN
#define USER_GPI_1_MOD	DA7280_SINGLE_PTN
#define USER_GPI_2_MOD	DA7280_SINGLE_PTN

#define USER_GPI_0_POL	DA7280_BOTH_EDGE
#define USER_GPI_1_POL	DA7280_BOTH_EDGE
#define USER_GPI_2_POL	DA7280_BOTH_EDGE


/*
 * End of Customer Platform Definitions.
 */


/*
* Function list
*/

int da7280_init(void);
int da7280_i2c_init(void);
int da7280_reg_read(u8 reg);
int da7280_reg_write(u8 reg, u8 val);
int da7280_set_seq_loop(u8 seq_loop);
int da7280_set_seq_id(u8 seq_id);
int da7280_haptic_enable(void);
int da7280_haptic_disable(void);
int da7280_haptic_mem_update(u8 *snp_mem, u8 size);
int da7280_haptic_mem_read(u8 *snp_mem, u8 size);
int da7280_set_dev_type(enum da7280_haptic_dev_t type);
u8 da7280_get_op_mode(void);
int da7280_set_op_mode(enum da7280_op_mode mode);
int da7280_bemf_sense_enable(u8 enable);
int da7280_freq_track_enable(u8 enable);
int da7280_acc_enable(u8 enable);
int da7280_amp_pid_enable(u8 enable);
int da7280_rapid_stop_enable(u8 enable);
int da7280_set_override_val(u8 val);
int da7280_set_gpi_seq_id(u8 gpi_num, u8 val);
int da7280_set_gpi_mod(u8 gpi_num, u8 val);
int da7280_set_gpi_pol(u8 gpi_num, u8 val);
int da7280_set_idac_gain(u16 val);
int da7280_set_resonant_freq(u16 val);
int da7280_set_imax(int val);
int da7280_set_volt_rating(u8 reg, u32 val);
int da7280_set_user_data(void);
int da7280_resume(void);
int da7280_suspend(void);
int da7280_set_default(void);
int da7280_irq_handler(void);
void dump_all_registers(char *phase);

#endif /* _DA7280_HAL_H */
