## Method comparison & limiations
To complete this project, I had two pathways I could've chosen.

1. Full smoke simulations

This method would likely have given the most accurate results, and most closely follows BOM's methothodology. I was able to code the simulation in rust, but it ended up being extremely expensive to run - and it would've taken a lot longer to tune the hyperparameters to match real world obersations (which I still have none of). While I did not choose this route, I don't doubt that it may still be viable - but there will be some issues with fires that were spotted late, requiring more compute.  

2. WESG10

This method was already implimented in python by [glasstone](https://github.com/GOFAI/glasstone), and being much cheaper than method 1, I was almost forced into this route. It does come with some problems, mainly that it doesnt really take into account changing wind directions/strengths, or the fact that it was meant for "impulse" explosions, rather than slowly burning bush. But while there are some limitations, it is far better than nothing and is much easier than method 1; so that's why this project is based off it.

## Energy

WESG10 expects the fire to be in units of megatons. Obviously, because fires are not nuclear weapons of mass destruction, some approximations had to be done.

1. Energy per hectare

Based on some really rough napkin math, I estimated that 3300 Ha is equiv. to 1 MT. Not going to show my working because I am certain the following factors influence it more.

2. Temperature

It made sense to me that hotter air/wind temperatures increases the amount of bush that could be burned, so I multiply the estimated energy based on the size of the fire by: $1.01^{temp-25}$. This formula is completely arbitrary, but is based on my personal expectations on how fires get exponentially worse as the temperature increases.

3. Out of control

Simularly, there's some [research](https://www.epa.nsw.gov.au/your-environment/air/open-burning-reducing-pollution) that supports that fires which are out of control tend to produce a lot more smoke. In my case, out of control fires increase the "fire energy" by 5. Again, arbitrary, and can be tuned for more accurate results.

4. Scaling

Because the WESG10 model expects a massive singular impluse, the reach of the resulting fallout tends to be quite far as the initial radiation density is quite high. For this reason, I scale the contours down by 30x to counteract this. Again, arbitrarily.

5. Contours

I selected the contours for 200, 1500, and 5000 roentgen. Because we are dealing with smoke, I just scaled these values down by 10000x, converted them to mSv, and called it a day. The is the most arbitrary calculation so far, but there were some reports saying how the worst of the sydney bushfires are about the same as smoking 20 cigarettes, or very roughly 53 mSv. If I am within a magnitude of error, thats close enough for me.

## Improving

Obviously, there is a lot more more art than science here, so feel free to create a github issue to finetune these variables if you can reason for it.