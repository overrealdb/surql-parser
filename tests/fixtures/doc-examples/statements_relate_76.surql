CREATE planet:venus, telescope:one;

RELATE telescope:one->observed:[d'2025-04-24T02:02:18.204Z']->planet:venus CONTENT { 
      temperature_profile: {
        surface: 735.0,
        upper_atmosphere: 300.0
      },
      composition: {
        CO2: 96.5,
        N2: 3.5,
        SO2: 0.015
      },
};

RELATE telescope:one->observed:[d'2025-04-25T02:02:18.204Z']->planet:venus CONTENT {
      temperature_profile: {
        surface: 737.0,
        upper_atmosphere: 298.5
      },
      composition: {
        CO2: 96.6,
        N2: 3.4,
        SO2: 0.015
    }
};

SELECT id, (<-observed:[d'2025-04-24']..).{
    at: id[0], 
    surface: temperature_profile.surface,
    atmosphere: temperature_profile.upper_atmosphere
} AS observations FROM planet;
