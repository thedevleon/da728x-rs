/*
 * hal-da7280.c - da7280 Haptic Hardware Abstraction Layer functions
 *
 * Copyright (C) 2017 Dialog Semiconductor Ltd. and its Affiliates, unpublished
 * work. This computer program includes Confidential, Proprietary Information
 * and is a Trade Secret of Dialog Semiconductor Ltd. and its Affiliates.
 * All use, disclosure, and/or reproduction is prohibited unless authorized
 * in writing. All Rights Reserved.
 */

#include "da7280_hal.h"

/*
 * Global Variables
 */

const struct scr_type da7280_pdata_setup[] = {

	/* Event clear */
	{DA7280_IRQ_EVENT1, 0xff},
	{DA7280_TOP_CTL2, USER_OVERIDE_VAL},
	{DA7280_SEQ_CTL2,
		USE_SEQ_LOOP << DA7280_PS_SEQ_LOOP_SHIFT |
		USE_SEQ_ID << DA7280_PS_SEQ_ID_SHIFT},
#ifdef DA7280_USER_GPIO
	{DA7280_GPI_0_CTL,
		USER_GPI_0_SEQ_ID << DA7280_GPI0_SEQUENCE_ID_SHIFT |
		USER_GPI_0_MOD << DA7280_GPI0_MODE_SHIFT |
		USER_GPI_0_POL << DA7280_GPI0_POLARITY_SHIFT},
	{DA7280_GPI_1_CTL,
		USER_GPI_1_SEQ_ID << DA7280_GPI0_SEQUENCE_ID_SHIFT |
		USER_GPI_1_MOD << DA7280_GPI0_MODE_SHIFT |
		USER_GPI_1_POL << DA7280_GPI0_POLARITY_SHIFT},
	{DA7280_GPI_2_CTL,
		USER_GPI_1_SEQ_ID << DA7280_GPI0_SEQUENCE_ID_SHIFT |
		USER_GPI_1_MOD << DA7280_GPI0_MODE_SHIFT |
		USER_GPI_1_POL << DA7280_GPI0_POLARITY_SHIFT},
#endif
	{SEQ_END, 0}
};

static struct da7280_haptic haptic;

/*
 *  I/O control interface functions.
 */

/* I2C Functions, Speed limitation: up to 3.4MHz */
int da7280_i2c_init(void)
{
	diag_info("da7280_i2c_init\n");
#ifdef DIALOG_TEST_PLATFORM
	return i2c_init();
#else  /* Arduino */
	/* Customer must add the i2c init function of target here
	  */
	return -EIO;
#endif
}

int da7280_reg_read(u8 reg)
{
#ifdef DIALOG_TEST_PLATFORM
	u8 val = 0;
	int ret;

	ret = i2c_register_read(reg, &val);
	if (ret < 0) {
		diag_err("i2c read error(%d) reg[0x%02x\n",
			ret, reg);
		return -EIO;
	}
	diag_i2c_read("Read [0x%02x]  = 0x%02x\n", reg, val);
	return val;
#else  /* Arduino */
	/* Customer must add the i2c read function of target here
	  */
	return -EIO;
#endif
}

int da7280_reg_write(u8 reg, u8 val)
{
#ifdef DIALOG_TEST_PLATFORM
	int ret = i2c_register_write(reg, val);

	if (ret) {
		diag_err("i2c write error(%d) [0x%02x 0x%02x]\n",
			ret, reg, val);
		return ret;
	}
	diag_i2c("Write [0x%02x] = 0x%02x\n", reg, val);
	return 0;
#else /* Arduino */
	/* Customer must add the i2c write function of target here
	  */
	return -EIO;
#endif
}

int da7280_reg_bulk_write(u8 reg, u8 *val, int val_count)
{
	int ret = 0, i;

	for (i = 0; i < val_count; i++) {
		ret = da7280_reg_write(reg + i, *val++);
		if (ret != 0)
			goto err;
	}
	return 0;
err:
	diag_err("error in reg bulk write\n");
	return -EIO;
}

int da7280_update_bits(u8 reg, u8 mask, u8 bits)
{
	u8 val = da7280_reg_read(reg);

	if (val < 0)
		return -EIO;

	val = val & ~mask; val |= bits;
	if (da7280_reg_write(reg, val) < 0)
		return -EIO;
	return 0;
}

static int da7280_run_script(const struct scr_type script[])
{
	int i;

	for (i = 0; script[i].reg != SEQ_END; i++) {
		if (script[i].reg == DA7280_DELAY) {
			msleep(script[i].val);
		} else if (da7280_reg_write((u8)script[i].reg, script[i].val)) {
			diag_err("script error in reg write\n");
			return -EIO;
		}
	}
	return 0;
}

static int da7280_run_script_mask(const struct scr_mask_type script[])
{
	int i;

	for (i = 0; script[i].reg != SEQ_END; i++) {
		if (script[i].reg == DA7280_DELAY) {
			msleep(script[i].val);
		} else if (da7280_update_bits((u8)script[i].reg,
			script[i].mask, script[i].val)) {
			diag_err("mask script error in reg write\n");
			return -EIO;
		}
	}
	return 0;
}

int da7280_haptic_mem_update(u8 *snp_mem, u8 size)
{
	int val;

	diag_info("da7280_haptic_mem_update\n");

	if (size > DA7280_SNP_MEM_SIZE) {
		diag_err("Invalid size\n");
		return -EINVAL;
	}

	/* It is recommended to update the patterns
	* during haptic is not working in order to avoid conflict
	*/
	val = da7280_reg_read(DA7280_IRQ_STATUS1);
	if (val < 0)
		return val;
	if (val & DA7280_STA_WARNING_MASK) {
		diag_warn("Warning! Please check HAPTIC status\n");
		return -EBUSY;
	}

	/* Patterns are not updated if the lock bit is enabled */
	val = 0;
	val = da7280_reg_read(DA7280_MEM_CTL2);
	if (val < 0)
		return val;
	if (~val & DA7280_PATTERN_MEM_LOCK_MASK) {
		diag_warn("Memory is locked. please unlock the bit first.\n");
		return -EACCES;
	}

	val = da7280_reg_read(DA7280_MEM_CTL1);
	if (val < 0)
		return val;
	return da7280_reg_bulk_write(val,
		snp_mem, DA7280_SNP_MEM_MAX - val + 1);
}

int da7280_haptic_mem_read(u8 *snp_mem, u8 size)
{
	int val, i, mem_val;

	if (size > DA7280_SNP_MEM_SIZE) {
		diag_err("Invalid size\n");
		return -EINVAL;
	}

	val = da7280_reg_read(DA7280_MEM_CTL1);
	if (val < 0)
		return val;

	for (i = 0; i < size; i++) {
		mem_val = da7280_reg_read(val + i);
		if (val < 0)
			return val;
		*(snp_mem + i) = (u8)mem_val & 0xff;
	}

	return 0;
}


int da7280_set_override_val(u8 val)
{
	u8 mask = 0xFF;

	if (val > mask) {
		diag_err("Invalid override value\n");
		return -EINVAL;
	}

	/* If acc_en == 1,
	*	set from 0 ~ 127, control level but not direction
	*   If acc_en == 0,
	*	set from -128 ~ 127, control level & direction
	*/
	if (haptic.acc_en ||
		(haptic.dev_type == DA7280_LRA))
		mask = 0x7F;

	if (val > mask)
		val = mask;

	/* Set driving level */
	return da7280_reg_write(DA7280_TOP_CTL2, val & mask);
}

int da7280_set_gpi_seq_id(u8 gpi_num, u8 val)
{
	if (val >= USER_SEQ_ID_MAX) {
		diag_err("Invalid value\n");
		return -EINVAL;
	}
	return da7280_update_bits(DA7280_GPI_0_CTL + gpi_num,
			DA7280_GPI0_SEQUENCE_ID_MASK,
			val << DA7280_GPI0_SEQUENCE_ID_SHIFT);
}

int da7280_set_gpi_mod(u8 gpi_num, u8 val)
{
	if (val >= DA7280_GPI_MOD_MAX) {
		diag_err("Invalid value\n");
		return -EINVAL;
	}
	return da7280_update_bits(DA7280_GPI_0_CTL + gpi_num,
			DA7280_GPI0_MODE_MASK,
			val << DA7280_GPI0_MODE_SHIFT);
}

int da7280_set_gpi_pol(u8 gpi_num, u8 val)
{
	if (val >= DA7280_GPI_POL_MAX) {
		diag_err("Invalid value\n");
		return -EINVAL;
	}
	return da7280_update_bits(DA7280_GPI_0_CTL + gpi_num,
			DA7280_GPI0_POLARITY_MASK,
			val << DA7280_GPI0_POLARITY_SHIFT);
}

/*
 * Use in case of LRA_MODE, default 180 Hz.
 *	the freq range: 50Hz ~ 300Hz.
 *
 * MS-bits of the initial LRA resonance frequency period Used
 * for specifying the LRA drive frequency
 *
 */
int da7280_set_resonant_freq(u16 val)
{
	int ret;
	u32 get_val;

	if (val > DA7280_MAX_RESONAT_FREQ ||
		val < DA7280_MIN_RESONAT_FREQ) {
		diag_err("Invalid value\n");
		return -EINVAL;
	}
	get_val = 1000000000 / (val * 1333);
	ret = da7280_reg_write(DA7280_FRQ_LRA_PER_H,
			(get_val >> 7) & 0xFF);
	if (ret)
		goto err;

	ret = da7280_reg_write(DA7280_FRQ_LRA_PER_L,
			get_val & 0x7F);
	if (ret)
		goto err;

	return 0;
err:
	diag_info("Error in da7280_set_resonant_freq: %d\n", ret);
	return ret;

}

int da7280_set_dev_type(enum da7280_haptic_dev_t type)
{
	if (type >= DA7280_DEV_MAX) {
		diag_err("Invalid type\n");
		return -EINVAL;
	}
	return da7280_update_bits(DA7280_TOP_CFG1,
			DA7280_ACTUATOR_TYPE_MASK,
			(type ? 1:0) << DA7280_ACTUATOR_TYPE_SHIFT);
}

int da7280_bemf_sense_enable(u8 enable)
{
	return da7280_update_bits(DA7280_TOP_CFG1,
		DA7280_BEMF_SENSE_EN_MASK,
		(enable ? 1:0) << DA7280_BEMF_SENSE_EN_SHIFT);
}

int da7280_freq_track_enable(u8 enable)
{
	return da7280_update_bits(DA7280_TOP_CFG1,
		DA7280_FREQ_TRACK_EN_MASK,
		(enable ? 1:0) << DA7280_FREQ_TRACK_EN_SHIFT);
}

int da7280_acc_enable(u8 enable)
{
	return da7280_update_bits(DA7280_TOP_CFG1,
		DA7280_ACCELERATION_EN_MASK,
		(enable ? 1:0) << DA7280_ACCELERATION_EN_SHIFT);
}

int da7280_rapid_stop_enable(u8 enable)
{
	return da7280_update_bits(DA7280_TOP_CFG1,
		DA7280_RAPID_STOP_EN_MASK,
		(enable ? 1:0) << DA7280_RAPID_STOP_EN_SHIFT);
}

int da7280_amp_pid_enable(u8 enable)
{
	return da7280_update_bits(DA7280_TOP_CFG1,
		DA7280_AMP_PID_EN_MASK,
		(enable ? 1:0) << DA7280_AMP_PID_EN_SHIFT);
}

int da7280_impd_check(int impd)
{
	if (impd > DA7280_IMPD_MAX
		|| impd < DA7280_IMPD_MIN) {
		diag_err("Invalid Impedance value\n");
		return -EINVAL;
	}
	return 0;
}

int da7280_set_imax(int val)
{
	int imax, ret;
	u32 v2i_factor;

	if (val > DA7280_IMAX_LIMIT) {
		diag_err("Invalid value\n");
		return -EINVAL;
	}
	imax = (val * 1000 - 28600) / DA7280_IMAX_STEP + 1;
	if (imax > 0x1F)
		imax = 0x1F;
	ret = da7280_update_bits(DA7280_ACTUATOR3,
		DA7280_IMAX_MASK,
		imax & DA7280_IMAX_MASK);
	if (ret)
		return ret;

	/* Impedance range check */
	ret = da7280_impd_check(USER_IMPD_mOhm);
	if (ret)
		return ret;

	v2i_factor = USER_IMPD_mOhm * 1000 * (imax + 4)
				/ 1610400;
	ret = da7280_reg_write(DA7280_CALIB_V2I_L,
		v2i_factor & 0xFF);
	if (ret)
		return ret;
	ret = da7280_reg_write(DA7280_CALIB_V2I_H,
		(v2i_factor >> 8) & 0xFF);
	return ret;
}

int da7280_set_volt_rating(u8 reg, u32 val)
{
	u32 voltage;

	if (val < DA7280_VOLT_RATE_MAX)
		voltage = (val * 1000 / DA7280_VOLT_STEP_uV + 1);
	else {
		voltage = DA7280_NOM_VOLT_DFT;
		diag_info("Set to default value");
	}

	if (voltage > 0xFF)
		voltage = 0xFF;

	return da7280_reg_write(reg, voltage & 0xFF);
}

int da7280_set_seq_id(u8 seq_id)
{
	if (seq_id > USER_SEQ_ID_MAX) {
		diag_err("Invalid value\n");
		return -EINVAL;
	}
	return da7280_update_bits(DA7280_SEQ_CTL2,
			DA7280_PS_SEQ_ID_MASK,
			seq_id << DA7280_PS_SEQ_ID_SHIFT);
}

int da7280_set_seq_loop(u8 seq_loop)
{
	if (seq_loop > USER_SEQ_LOOP_MAX) {
		diag_err("Invalid value\n");
		return -EINVAL;
	}
	return da7280_update_bits(DA7280_SEQ_CTL2,
			DA7280_PS_SEQ_LOOP_MASK,
			seq_loop << DA7280_PS_SEQ_LOOP_SHIFT);
}

int da7280_set_op_mode(enum da7280_op_mode mode)
{
	if (mode >= DA7280_MODE_MAX || mode < 0) {
		diag_err("Invalid mode\n");
		return -EINVAL;
	}

	haptic.op_mode = mode;
	diag_info("Set op mode to (%d)\n", mode);
	return 0;
}

u8 da7280_get_op_mode(void)
{
	return haptic.op_mode;
}

static u8 da7280_set_pwm(void)
{
#ifdef DA7280_HAPTIC_PWM
	return 0;
#else
	/* pwm handling code here. */
	diag_info("PWM is not supported now\n");
	return -EOPNOTSUPP;
#endif
}

int da7280_pwm_check(int freq_hz, int pwm_duty)
{
	if (freq_hz < DA7280_MIN_PWM_FREQ_kHz
		|| freq_hz > DA7280_MAX_PWM_FREQ_kHz) {
		diag_err("Invalid freq range");
		return -EINVAL;
	}

	/* pwm_duty range.
	 *	1. Full range (0 ~ 100%) when ACCELERATION_EN = 1.
	 *	2. Half range (50 ~ 100%)  when ACCELERATION_EN = 0.
	 */
	if (haptic.acc_en == 0 && pwm_duty < 50 /* % */) {
		diag_err("Invalid freq range");
		return -EINVAL;
	}
	return 0;
}

static int da7280_pwm_disable(void)
{
#ifdef DA7280_HAPTIC_PWM
	/* pwm handling code here. */
#else
	diag_info("PWM is not supported now\n");
#endif
	return 0;
}

/*
 * da7280_haptic_enable:	Haptic will start driving
 *	according to below modes set in advance.
 *
 * In case of DA7280_DRO_MODE,
 *	User is able to change the overide val to change
 *	The drive level of the output by
 *	da7280_set_override_val function call
 *	before/after da7280_haptic_enable function call like
 *		da7280_set_override_val(120);
 *
 * In case of DA7280_PWM_MODE,
 *	The pwm signal has to be generated by host
 *	"BEFORE" setting the op mode to pwm mode,
 *	which means
 *		changing OPERATION_MODE to 2 (PWM mode)
 *	Please add some code the target platform provides in
 *	da7280_set_pwm function for successful operation.
 *	Then, just call the function below for pwm mode operation.
 *	i.e.
 *		da7280_haptic_enable();
 *
 * In case of DA7280_RTWM_MODE,
 *	User is able to change the pattern by
 *	da7280_set_seq_id/da7280_set_seq_loop function
 *	before da7280_haptic_enable function call.
 *	i.e.
 *		da7280_set_seq_id(7);
 *		da7280_set_seq_loop(3);
 *		da7280_haptic_enable();
 *
 * In case of DA7280_ETWM_MODE,
 *	User is able to change the pattern or ways to trigger by
 *	da7280_set_gpi_seq_id/mod/pol functions
 *	before da7280_haptic_enable function call.
 *	i.e.
 *		da7280_set_gpi_seq_id(0, 7);
 *		da7280_set_gpi_seq_id(1, 7);
 *		da7280_set_gpi_seq_id(2, 7);
 *		da7280_haptic_enable();
 */
int da7280_haptic_enable(void)
{
	int ret = 0;
	u8 mask = 0xFF;

	diag_info("haptic_enable\n");

	if (haptic.op_mode == DA7280_PWM_MODE) {
		ret = da7280_set_pwm();
		if (ret)
			goto err;
	}

	ret = da7280_update_bits(DA7280_TOP_CTL1,
			DA7280_OPERATION_MODE_MASK,
			haptic.op_mode << DA7280_OPERATION_MODE_SHIFT);
	if (ret)
		goto err;

	if (haptic.op_mode == DA7280_PWM_MODE
		|| haptic.op_mode == DA7280_RTWM_MODE) {
		diag_info("Set SEQ_START\n");
		ret = da7280_update_bits(DA7280_TOP_CTL1,
			DA7280_SEQ_START_MASK,
			DA7280_SEQ_START_MASK);
		if (ret)
			goto err;
	}
	return 0;

err:
	diag_err("Error in da7280_haptic_enable : %d\n", ret);
	return ret;
}

int da7280_haptic_disable(void)
{
	int ret;

	diag_info("da7280_haptic_disable\n");

	/* In case of DA7280_PWM_MODE,
	** external PWM signal must be set off after this function is called
	** Otherwise, some error may happen.
	*/
	ret = da7280_update_bits(DA7280_TOP_CTL1,
		DA7280_OPERATION_MODE_MASK, 0);
	if (ret)
		goto err;
	/* Then, please disable the pwm signal here in case of pwm mode */
	if (haptic.op_mode == DA7280_PWM_MODE) {
		diag_info("da7280 pwm disable\n");
		ret = da7280_pwm_disable();
		if (ret)
			goto err;
	}
	return 0;

err:
	diag_err("Error in da7280_haptic_disable : %d\n", ret);
	return ret;
}

int da7280_irq_handler(void)
{
	u8 events[IRQ_NUM];
	int ret, val, i;

	/* Check what events have happened */
	for (i = 0; i < 3; i++) {
		val = da7280_reg_read(DA7280_IRQ_EVENT1 + i);

		if (val < 0) {
			ret = val;
			goto err;
		}
		events[i] = val;
	}

	/* Empty check due to shared interrupt */
	if ((events[0] | events[1] | events[2]) == 0x00)
		return 0;

	if (events[0] & DA7280_E_PAT_FAULT_MASK) {
		/* Stop first if Haptic is working
		 * Otherwise, the fault may happen continually
		* even though the bit is cleared.
		*/
		ret = da7280_update_bits(
				DA7280_TOP_CTL1,
				DA7280_OPERATION_MODE_MASK, 0);
		if (ret)
			goto err;
	}

	/* Clear events */
	ret = da7280_reg_write(
			DA7280_IRQ_EVENT1, events[0]);
	if (ret)
		goto err;

	/* Event handling for DA7280_IRQ_EVENT1 */
	for (i = 0; i < IRQ_NUM; i++) {
		if (events[i])
			diag_info("da7280-haptic event(%d): 0x%x\n",
					i, events[i]);
	}
	return 0;

err:
	diag_err("DA7280 haptic irq error : %d\n", ret);
	return ret;
}

int da7280_irq_status(void)
{
	return da7280_reg_read(DA7280_IRQ_STATUS1);
}

/* set to suspend mode */
int da7280_suspend(void)
{
	int ret;

	diag_info("da7280_suspend\n");
	if (haptic.suspend_state) {
		diag_info("It's already suspend mode\n");
		return 0;
	}

	ret = da7280_update_bits(DA7280_TOP_CTL1,
			DA7280_STANDBY_EN_MASK, 0);
	if (ret)
		return ret;
	haptic.suspend_state = true;

	return 0;
}

/* set to standby mode */
int da7280_resume(void)
{
	int ret;

	diag_info("da7280_resume\n");
	if (haptic.suspend_state == 0) {
		diag_info("It's already resume mode\n");
		return 0;
	}
	ret = da7280_update_bits(DA7280_TOP_CTL1,
		DA7280_STANDBY_EN_MASK,
		DA7280_STANDBY_EN_MASK);
	if (ret) {
		diag_err("I2C error : %d\n", ret);
		return ret;
	}
	haptic.suspend_state = false;
	return 0;
}

/* Initialisation da7280 */
int da7280_init(void)
{
	diag_info("da7280_init\n");

	/* Apply user data and default data */
	if (da7280_set_user_data())
		goto err;

	/* User specific initialisation code here if need.*/

	haptic.suspend_state = 1;
	if (da7280_resume())
		goto err;
	return 0;
err:
	diag_err("da7280_init error\n");
	return -EIO;
}

/* Platform depencecies */
int da7280_set_user_data(void)
{
	int i = 0;
	int ret;
	u8 mask, val;

	diag_info("da7280_set_user_data\n");
	haptic.dev_type = USER_HAPTIC_DEV;
	haptic.op_mode = USER_OP_MODE;
	haptic.bemf_sense_en = USER_BEMF_SENS_EN;
	haptic.freq_track_en = USER_FREQ_TRACK_EN;
	haptic.acc_en = USER_ACC_EN;
	haptic.rapid_stop_en = USER_RAPID_STOP_EN;
	haptic.amp_pid_en = USER_AMP_PID_EN;

	switch (haptic.dev_type) {
	case DA7280_LRA:
		ret = da7280_set_resonant_freq(USER_RESONANT_FREQ_Hz);
		if (ret)
			return ret;
		break;
	case DA7280_ERM_COIN:
		ret = da7280_update_bits(DA7280_TOP_INT_CFG1,
			DA7280_BEMF_FAULT_LIM_MASK, 0);
		if (ret)
			return ret;

		ret = da7280_update_bits(DA7280_TOP_CFG4,
			DA7280_TST_CALIB_IMPEDANCE_DIS_MASK |
			DA7280_V2I_FACTOR_FREEZE_MASK,
			DA7280_TST_CALIB_IMPEDANCE_DIS_MASK |
			DA7280_V2I_FACTOR_FREEZE_MASK);
		if (ret)
			return ret;

		haptic.acc_en = 0;
		haptic.rapid_stop_en = 0;
		haptic.amp_pid_en = 0;
		break;
	default:
		break;
	}

	if (haptic.op_mode >= DA7280_RTWM_MODE)
		haptic.bemf_sense_en = 0;

	/* Set actuator type(LRA/ERM) and several bits in DA7280_TOP_CFG1
	*/
	mask = DA7280_ACTUATOR_TYPE_MASK |
			DA7280_BEMF_SENSE_EN_MASK |
			DA7280_FREQ_TRACK_EN_MASK |
			DA7280_ACCELERATION_EN_MASK |
			DA7280_RAPID_STOP_EN_MASK |
			DA7280_AMP_PID_EN_MASK;

	val = (haptic.dev_type ? 1:0) << DA7280_ACTUATOR_TYPE_SHIFT |
		(haptic.bemf_sense_en ? 1:0) << DA7280_BEMF_SENSE_EN_SHIFT |
		(haptic.freq_track_en ? 1:0) << DA7280_FREQ_TRACK_EN_SHIFT |
		(haptic.acc_en ? 1:0) << DA7280_ACCELERATION_EN_SHIFT |
		(haptic.rapid_stop_en ? 1:0) << DA7280_RAPID_STOP_EN_SHIFT |
		(haptic.amp_pid_en ? 1:0) << DA7280_AMP_PID_EN_SHIFT;

	ret = da7280_update_bits(DA7280_TOP_CFG1, mask, val);
	if (ret)
		return ret;

	ret = da7280_set_imax(USER_IMAX_mA);
	if (ret)
		return ret;
	/* set nom_volt_rating */
	ret = da7280_set_volt_rating(DA7280_ACTUATOR1, USER_NOM_mVolt);
	if (ret)
		return ret;
	/* set abs_overdrive_volt */
	ret = da7280_set_volt_rating(DA7280_ACTUATOR2, USER_ABS_MAX_mV);
	if (ret)
		return ret;

	if (da7280_run_script(da7280_pdata_setup)) {
		diag_err("da7280_set_user_data error\n");
		return -EINVAL;
	}
	return 0;
}

/* Set registers to default values */
int da7280_set_default(void)
{
	int ret = da7280_set_user_data();

	diag_info("da7280_set_default\n");
	if (ret)
		return ret;
	return 0;
}

/*
 * DEBUG & TEST Functions
 */

 /* Read back all registers and print out */
void dump_all_registers(char *phase)
{
#ifdef DA7280_DEBUG
	int i, j;

	diag_info("%s\n", phase);
	diag_info("reg[--..] = 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n");
	for (j = 0; j < 59; j++)
		diag_info("-");

	for (j = 0; j < 16; j++) {
		if (j == 0)
			diag_info("\nreg[00..] = ");
		else
			diag_info("reg[%x..] = ", j * 16);

		for (i = 0; i < 16; i++) {
			u8 reg = da7280_reg_read(j * 16 + i);

			if (reg < 0x10)
				diag_info("0%x ", reg);
			else
				diag_info("%x ", (reg));
		}
		diag_info("\n");
	}
#else
	return;
#endif
}
